//! Native standalone-sharing investigation. All operations use bounded virtual state.
use super::*;
use dlms_learning_lab::{samba_share_source as binding, samba_share_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub split: bool,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Rect, egui::Id)>,
    pub results: Option<egui::Rect>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn select<T: Clone + PartialEq>(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    value: &mut T,
    options: &[(T, &str)],
    m: &mut Metrics,
) {
    controls::form_row(ui, label, |ui, width| {
        let selected = options
            .iter()
            .find(|(v, _)| v == value)
            .map_or("Choose a supplied value", |(_, s)| *s);
        let r = controls::dropdown(ui, key, width)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let begin = m.controls.len();
                for (v, label) in options {
                    let r = ui.selectable_value(value, v.clone(), *label);
                    m.record(ui, &format!("{key}:{label}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for r in &mut m.controls[begin..] {
                    r.2 = clip;
                }
            });
        m.record(ui, key, &r.response);
    });
}
fn accounts(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &mut work::Attempt,
    m: &mut Metrics,
) -> Result<(), String> {
    ui.strong("Supplied SMB accounts");
    lesson_presentation::prose(
        ui,
        "Simulator administration · these controls enable supplied identities; they do not set real passwords. Unix groups remain separate.",
    );
    for goal in &t.accounts {
        let account = a.world.accounts.iter().find(|u| u.user == goal.user);
        let state = match account {
            None => "Not provisioned",
            Some(u) if u.enabled => "Enabled",
            _ => "Disabled",
        };
        let enabled = account.is_some_and(|u| u.enabled);
        controls::form_row(ui, goal.user.as_str(), |ui, _| {
            ui.horizontal_wrapped(|ui| {
                let width =
                    (ui.text_style_height(&egui::TextStyle::Body) * 8.).min(ui.available_width());
                controls::cell(ui, width, |ui| {
                    ui.label(state);
                });
                if t.editable_accounts.iter().any(|u| u.id == goal.user) {
                    let r = ui.add(button(if enabled { "Disable" } else { "Add / enable" }));
                    m.record(ui, &format!("account:{}", goal.user), &r);
                    if r.clicked() {
                        a.account(t, &goal.user, !enabled)?;
                    }
                }
                Ok::<_, String>(())
            })
            .inner
        })
        .inner?;
    }
    let r = controls::collapsing_header(ui, "Inspect supplied Unix identities and groups").show(
        ui,
        |ui| {
            for (user, groups) in &a.world.files.users {
                lesson_presentation::monospace(ui, &format!("{user} · {}", groups.join(" ")), true);
            }
        },
    );
    m.record(ui, "identities", &r.header_response);
    Ok(())
}
fn editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &mut work::Attempt,
    m: &mut Metrics,
    scope: egui::Id,
) -> Result<(), String> {
    ui.strong("Share configuration");
    lesson_presentation::monospace(ui, &a.world.configuration_path, true);
    let saved = a.world.saved_config().map_err(|e| e.detail)?;
    lesson_presentation::prose(
        ui,
        &format!(
            "Editor: {} · Saved and applied text: {}",
            if a.editor == saved {
                "saved"
            } else {
                "unsaved changes"
            },
            if saved == a.world.published {
                "same"
            } else {
                "different"
            }
        ),
    );
    let mut text = a.editor.clone();
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 11. + 2. * controls::PAD_Y as f32;
    let out = controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt("smb-editor-scroll")
            .max_height(h)
            .min_scrolled_height(h),
        |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut text)
                    .code_editor()
                    .lock_focus(false)
                    .font(egui::TextStyle::Monospace)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_width((ui.available_width() - 2. * controls::PAD_X as f32).max(24.))
                    .desired_rows(11)
                    .char_limit(16384)
                    .id(scope.with("config")),
            )
        },
    );
    m.editor = Some((
        out.inner.rect,
        out.inner_rect.intersect(ui.clip_rect()),
        out.inner.id,
    ));
    m.controls
        .push(("config".into(), out.inner_rect, ui.clip_rect()));
    if text != a.editor {
        a.edit(t, text)?;
    }
    ui.horizontal_wrapped(|ui|{
        let r=ui.add(button("Save configuration"));m.record(ui,"save",&r);if r.clicked(){a.save(t)?;a.record("Saved editor text to the virtual configuration file. Apply saved settings before testing a policy change.".into());}
        let r=ui.add(button("Check syntax"));m.record(ui,"syntax",&r);if r.clicked(){match a.world.check_saved(){Ok(_)=>a.record("Saved syntax is supported. This does not prove client access; run a trial.".into()),Err(e)=>a.record(format!("Syntax check: {}",e.detail))}}
        let r=ui.add(button("Apply saved settings"));m.record(ui,"apply",&r);if r.clicked(){a.publish(t)?;a.record("Applied saved settings. Each new trial opens a new simulated connection.".into());}
        Ok::<_,String>(())
    }).inner?;
    lesson_presentation::prose(
        ui,
        "Apply is simulator administration for new trials, not a Linux command.",
    );
    let r = controls::collapsing_header(ui, "Inspect settings currently used by new clients").show(
        ui,
        |ui| {
            lesson_presentation::monospace(ui, &a.world.published, true);
        },
    );
    m.record(ui, "applied", &r.header_response);
    Ok(())
}
fn terminal(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &mut work::Attempt,
    m: &mut Metrics,
    scope: egui::Id,
) {
    ui.strong("Filesystem terminal");
    lesson_presentation::prose(
        ui,
        "Inspect users and file permissions here. Supported: ls, id, cat, pwd, cd, chmod, chgrp.",
    );
    controls::form_row(ui, "Command", |ui, width| {
        let r = ui.add(
            controls::singleline_fitting(ui, &mut a.terminal, width)
                .hint_text("id -Gn USER or ls -ld PATH")
                .char_limit(2048)
                .id(scope.with("terminal")),
        );
        m.fields
            .push(("terminal".into(), r.rect, r.id, ui.clip_rect()));
        m.record(ui, "terminal", &r);
    });
    let r = ui.add(button("Run command"));
    m.record(ui, "command-run", &r);
    let enter = ui.input(|i| i.key_pressed(egui::Key::Enter))
        && ui.ctx().memory(|v| v.has_focus(scope.with("terminal")));
    if r.clicked() || enter {
        let command = a.terminal.clone();
        let response = a
            .command(t, &command)
            .unwrap_or_else(|e| format!("Command not applied: {e}"));
        a.record(format!("$ {command}\n{response}"));
    }
}
fn trial(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &mut work::Attempt,
    m: &mut Metrics,
    scope: egui::Id,
) -> Result<(), String> {
    ui.strong("Client trial");
    lesson_presentation::prose(
        ui,
        "Open a new simulated connection and read or replace an existing file. A trial changes only this workspace; Check answers tests the whole assignment.",
    );
    let before = a.request.clone();
    let clients: Vec<_> = t
        .clients
        .iter()
        .map(|c| (c.id.clone(), c.label.as_str()))
        .collect();
    select(ui, "client", "Client", &mut a.request.client, &clients, m);
    let shares: Vec<_> = t
        .shares
        .iter()
        .map(|s| (s.id.clone(), s.id.as_str()))
        .collect();
    select(ui, "share", "Share", &mut a.request.share, &shares, m);
    let mut write = matches!(a.request.operation, work::Operation::Replace { .. });
    select(
        ui,
        "operation",
        "Operation",
        &mut write,
        &[(false, "Read file"), (true, "Replace file")],
        m,
    );
    let mut path = match &a.request.operation {
        work::Operation::Read { path } | work::Operation::Replace { path, .. } => path.clone(),
    };
    controls::form_row(ui, "File in share", |ui, width| {
        let r = ui.add(
            controls::singleline_fitting(ui, &mut path, width)
                .hint_text("Relative file name")
                .char_limit(512)
                .id(scope.with("path")),
        );
        m.fields.push(("path".into(), r.rect, r.id, ui.clip_rect()));
        m.record(ui, "path", &r);
    });
    a.request.operation = if write {
        let mut text = match &a.request.operation {
            work::Operation::Replace { text, .. } => text.clone(),
            _ => String::new(),
        };
        ui.label("Replacement text · existing file only");
        let r = ui.add(
            egui::TextEdit::multiline(&mut text)
                .desired_rows(3)
                .desired_width((ui.available_width() - 24.).max(24.))
                .char_limit(4096)
                .id(scope.with("payload")),
        );
        m.fields
            .push(("payload".into(), r.rect, r.id, ui.clip_rect()));
        m.record(ui, "payload", &r);
        work::Operation::Replace { path, text }
    } else {
        work::Operation::Read { path }
    };
    if a.request != before {
        a.last_trial = None;
    }
    let r = ui.add(button("Run trial"));
    m.record(ui, "trial-run", &r);
    if r.clicked() {
        a.run(t)?;
    }
    let r = ui.strong("Actual trial result");
    m.record(ui, "trial-title", &r);
    if let Some(observed) = &a.last_trial {
        match &observed.result {
            Ok(work::Outcome::Read(r)) => {
                lesson_presentation::prose(ui, "Read succeeded");
                lesson_presentation::monospace(ui, &r.path, true);
                lesson_presentation::monospace(ui, &r.text, true);
            }
            Ok(work::Outcome::Replaced(r)) => {
                lesson_presentation::prose(
                    ui,
                    &format!("Replacement succeeded · {} bytes written", r.bytes),
                );
                lesson_presentation::monospace(ui, &r.path, true);
            }
            Err(e) => {
                lesson_presentation::prose(ui, &format!("Denied / not completed · {}", e.detail));
            }
        }
    } else {
        lesson_presentation::prose(
            ui,
            "Not run for the current client, operation and file. Choose Run trial to observe it.",
        );
    }
    Ok(())
}
fn results(
    ui: &mut egui::Ui,
    t: &work::Task,
    a: &work::Attempt,
    s: &Scenario,
    practice: bool,
    m: &mut Metrics,
) {
    if !a.checked {
        let r = lesson_presentation::prose(
            ui,
            "Unchecked · Check answers tests all required clients and preservation rules.",
        );
        m.record(ui, "unchecked", &r);
        return;
    }
    let checks = a.checks(t);
    let passed = checks.iter().filter(|c| c.passed).count();
    let ok = passed == checks.len();
    let r = ui.strong(if ok {
        if practice {
            "Example checks pass"
        } else {
            "Assignment complete"
        }
    } else {
        "Sharing task needs attention"
    });
    m.results = Some(r.rect);
    m.record(ui, "result-title", &r);
    lesson_presentation::prose(
        ui,
        &format!("{passed}/{} requirements and cases pass.", checks.len()),
    );
    for c in checks.iter().filter(|c| !c.passed) {
        let r =
            lesson_presentation::prose(ui, &format!("Needs attention · {}\n{}", c.label, c.detail));
        m.record(ui, "failed-case", &r);
    }
    let r = controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
        for c in checks.iter().filter(|c| c.passed) {
            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
        }
    });
    m.record(ui, "passed-cases", &r.header_response);
    if ok {
        lesson_presentation::prose(ui, &s.explanation);
    }
}
impl LabApp {
    pub(super) fn samba_share_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let s = self.scenarios[i].clone();
        let original = &self.progress.attempts[&s.id];
        if let Err(e) = binding::context(original, &s) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::SambaShare { definition }, engine::State::SambaShare { simulation }) =
            (&s.goal, &original.state)
        else {
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
                    self.metrics.samba.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &s, "", Vec::<(String, String)>::new());
            lesson_presentation::worked(
                ui,
                "Separate guide server · inspect the relevant layers, then try the repairs.",
            );
            let worked = ui.scope(|ui| lesson_presentation::worked_rows(ui, &s));
            self.metrics
                .samba
                .record(ui, "worked-table", &worked.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.samba.record(ui, "example", &r);
            self.metrics.example_action = Some(r.rect);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let t = if practice {
                &definition.example
            } else {
                &definition.assignment
            };
            let a = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            if practice {
                lesson_presentation::practice_step(ui, &s, 0);
            } else {
                lesson_presentation::assignment(ui, &s);
            }
            let scope = egui::Id::new(("smb-work", &s.id, practice));
            ui.push_id(scope, |ui| {
                ui.add_enabled_ui(active, |ui| {
                    let config_needed = t.initial.published != t.reference.published;
                    let main = |ui: &mut egui::Ui,
                                a: &mut work::Attempt,
                                m: &mut Metrics|
                     -> Result<(), String> {
                        if !t.editable_accounts.is_empty() {
                            accounts(ui, t, a, m)?;
                        }
                        if practice {
                            lesson_presentation::practice_step(ui, &s, 1);
                        }
                        if config_needed {
                            editor(ui, t, a, m, scope)?;
                        } else {
                            terminal(ui, t, a, m, scope);
                            let r = controls::collapsing_header(
                                ui,
                                "Inspect or edit share configuration",
                            )
                            .show(ui, |ui| editor(ui, t, a, m, scope));
                            m.record(ui, "config-help", &r.header_response);
                            if let Some(r) = r.body_returned {
                                r?;
                            }
                        }
                        Ok(())
                    };
                    // Reserve the actual shared label/status/action row in each
                    // column. The breakpoint follows font/control metrics and
                    // fits inside the app's content cap at ordinary desktop size.
                    let column = controls::form_label_width(ui)
                        + 2. * ui.spacing().item_spacing.x
                        + ui.text_style_height(&egui::TextStyle::Body) * 8.
                        + controls::button_width(ui, "Add / enable")
                        + 2. * controls::PAD_X as f32;
                    let split = ui.available_width() > 2. * column + ui.spacing().item_spacing.x;
                    self.metrics.samba.split = split;
                    let result = if split {
                        ui.columns(2, |cols| {
                            main(&mut cols[0], a, &mut self.metrics.samba)?;
                            if practice {
                                lesson_presentation::practice_step(&mut cols[1], &s, 2);
                            }
                            trial(&mut cols[1], t, a, &mut self.metrics.samba, scope)
                        })
                    } else {
                        main(ui, a, &mut self.metrics.samba).and_then(|()| {
                            if practice {
                                lesson_presentation::practice_step(ui, &s, 2);
                            }
                            trial(ui, t, a, &mut self.metrics.samba, scope)
                        })
                    };
                    if let Err(e) = result {
                        self.notice = Some(e);
                    }
                    if config_needed {
                        let r = controls::collapsing_header(
                            ui,
                            "Investigate Unix file access in the terminal",
                        )
                        .show(ui, |ui| terminal(ui, t, a, &mut self.metrics.samba, scope));
                        self.metrics
                            .samba
                            .record(ui, "terminal-help", &r.header_response);
                    }
                    if !a.output.is_empty() {
                        let h = ui.text_style_height(&egui::TextStyle::Monospace) * 7.;
                        controls::nested_scroll(
                            ui,
                            egui::ScrollArea::vertical()
                                .id_salt("smb-history")
                                .max_height(h)
                                .stick_to_bottom(true),
                            |ui| {
                                for line in &a.output {
                                    lesson_presentation::monospace(ui, line, true);
                                }
                            },
                        );
                    }
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add_enabled(active, primary(ui, "Check answers"));
                        self.metrics.check = Some(r.rect);
                        self.metrics.samba.record(ui, "check", &r);
                        if r.clicked()
                            && let Err(e) = a.check(t)
                        {
                            self.notice = Some(e);
                        }
                        if practice {
                            if config_needed {
                                let r = ui.add(button("Fill worked guide configuration"));
                                self.metrics.samba.record(ui, "worked-config", &r);
                                if r.clicked()
                                    && let Err(e) = a.edit(
                                        t,
                                        t.reference.saved_config().unwrap_or_default().into(),
                                    )
                                {
                                    self.notice = Some(e);
                                }
                            }
                            let r = ui.add(button("Reset example only"));
                            self.metrics.samba.record(ui, "reset-example", &r);
                            if r.clicked() {
                                match work::Attempt::new(t) {
                                    Ok(reset) => *a = reset,
                                    Err(e) => self.notice = Some(e),
                                }
                            }
                        }
                    });
                    results(ui, t, a, &s, practice, &mut self.metrics.samba);
                });
            });
            ui.push_id(("smb-help", &s.id, challenge, practice), |ui| {
                let help =
                    controls::collapsing_header(ui, "Optional help and supported simulation scope")
                        .show(ui, |ui| {
                            if !practice {
                                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                                if ui
                                    .add_enabled(
                                        active && a.hints < s.hints.len(),
                                        button("Show a hint"),
                                    )
                                    .clicked()
                                {
                                    a.hints += 1;
                                    self.dirty = true;
                                }
                                for hint in s.hints.iter().take(a.hints) {
                                    lesson_presentation::prose(ui, hint);
                                }
                                controls::collapsing_header(
                                    ui,
                                    "Worked approach · reveals reasoning",
                                )
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, &s.explanation);
                                });
                            }
                            lesson_presentation::prose(ui, binding::HELP);
                            lesson_presentation::prose(ui, &s.simplifications);
                            for link in &s.sources {
                                ui.hyperlink_to(link, link);
                            }
                        });
                self.metrics.samba.record(ui, "help", &help.header_response);
            });
        }
        if active && next != before {
            match definition.context(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&s.id).unwrap().state =
                        engine::State::SambaShare {
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
