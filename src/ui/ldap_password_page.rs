//! Focused native client configuration and fictional password utility workspace.
use super::*;
use dlms_learning_lab::ldap_password_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub config: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub command: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub output: Option<egui::Rect>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn prompt(
    ui: &mut egui::Ui,
    label: &str,
    key: &str,
    value: &mut Option<work::Key>,
    t: &work::Task,
    m: &mut Metrics,
    allow_none: bool,
) {
    controls::form_row(ui, label, |ui, w| {
        let selected = value
            .as_ref()
            .and_then(|v| t.profiles.iter().find(|p| p.id == v.0))
            .map_or("Not supplied", |p| p.label.as_str());
        let response = controls::dropdown(ui, key, w)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                if allow_none {
                    let r = ui.selectable_value(value, None, "Not supplied");
                    record(m, format!("{key}-none"), &r, ui);
                }
                for p in &t.profiles {
                    let r = ui.selectable_value(value, Some(work::Key(p.id.clone())), &p.label);
                    record(m, format!("{key}-{}", p.id), &r, ui);
                }
            });
        record(m, key, &response.response, ui);
    });
}
fn inputs(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
    lesson: (&Scenario, bool),
    focus: &mut bool,
) {
    let (source, practice) = lesson;
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    let mut draft = s.draft.clone();
    lesson_presentation::monospace(
        ui,
        &format!(
            "Requested person: {}\nVirtual directory: {}",
            t.actor, t.uri
        ),
        true,
    );
    let case = &t.cases[usize::from(s.selected_case)];
    lesson_presentation::prose(
        ui,
        &format!(
            "Sample certificate issuer: {}. Current target password: {}.",
            case.certificate.issuer,
            if case.policy.require_old_password {
                "Required by this sample"
            } else {
                "Optional in this sample"
            }
        ),
    );
    ui.strong("Supplied CA inputs");
    for input in &t.ca_inputs {
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} · issuer {}",
                input.path,
                input
                    .roots
                    .iter()
                    .map(|r| r.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            true,
        );
    }
    ui.strong("Virtual client settings");
    let outer_width = ui.available_width();
    let h = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 4.
        + 2. * f32::from(controls::PAD_Y);
    controls::nested_scroll(
        ui,
        egui::ScrollArea::horizontal()
            .id_salt("password-client-horizontal")
            .max_width(outer_width)
            .min_scrolled_width(outer_width)
            .max_height(h)
            .auto_shrink([false, false]),
        |ui| {
            let out = egui::TextEdit::multiline(&mut draft.config)
                .code_editor()
                .lock_focus(false)
                .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                .desired_rows(4)
                .desired_width((outer_width - 2. * f32::from(controls::PAD_X)).max(24.))
                .id(egui::Id::new(("password-client", &t.id)))
                .char_limit(8192)
                .show(ui);
            m.config = Some((out.response.rect, out.response.id, ui.clip_rect()));
            record(m, "config", &out.response, ui);
        },
    );
    controls::form_row(ui, "Command", |ui, w| {
        let r = ui.add(
            controls::singleline_fitting(ui, &mut draft.command, w)
                .code_editor()
                .lock_focus(false)
                .id(egui::Id::new(("password-command", &t.id)))
                .hint_text("Literal directory utility command")
                .char_limit(4096),
        );
        m.command = Some((r.rect, r.id, ui.clip_rect()));
        record(m, "command", &r, ui);
        if *focus && ui.is_enabled() {
            r.request_focus();
            *focus = false;
        }
    });
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    let mut bind = Some(work::Key(draft.bind_profile.clone()));
    prompt(ui, "Log in (-W)", "bind", &mut bind, t, m, false);
    if let Some(bind) = bind {
        draft.bind_profile = bind.0;
    }
    prompt(
        ui,
        "Old target (-A)",
        "old",
        &mut draft.old_profile,
        t,
        m,
        true,
    );
    prompt(
        ui,
        "Replacement (-S)",
        "replacement",
        &mut draft.replacement_profile,
        t,
        m,
        true,
    );
    if draft != s.draft {
        *notice = s.edit(draft).err();
    }
}
pub(super) fn probe_text(
    ui: &egui::Ui,
    text: &str,
    width: f32,
    strong: bool,
    color: egui::Color32,
) -> std::sync::Arc<egui::Galley> {
    let text = RichText::new(text).color(color);
    let mut job = egui::WidgetText::from(if strong { text.strong() } else { text })
        .into_layout_job(ui.style(), egui::FontSelection::Default, egui::Align::LEFT)
        .as_ref()
        .clone();
    job.halign = egui::Align::LEFT;
    job.justify = false;
    job.wrap.max_width = width;
    ui.fonts_mut(|f| f.layout_job(job))
}
fn probe_cell(ui: &mut egui::Ui, text: &str, width: f32, strong: bool, color: egui::Color32) {
    let galley = probe_text(ui, text, width, strong, color);
    ui.add_sized([width, 0.], egui::Label::new(galley));
}
fn probe_table(ui: &mut egui::Ui, probes: &[work::Probe], salt: impl std::hash::Hash) {
    let status = |accepted| if accepted { "Accepted" } else { "Rejected" };
    let width = ui.available_width();
    if width < ui.text_style_height(&egui::TextStyle::Body) * 28. {
        for p in probes {
            lesson_presentation::prose(ui, &p.label);
            ui.horizontal_wrapped(|ui| {
                ui.label(format!("Expected: {}", status(p.expected)));
                lesson_presentation::rich(
                    ui,
                    RichText::new(format!("Actual: {}", status(p.accepted))).color(
                        if p.accepted == p.expected {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        },
                    ),
                );
            });
        }
    } else {
        let spacing = ui.spacing().item_spacing;
        let status_width =
            controls::button_width(ui, "Accepted").max(controls::button_width(ui, "Expected"));
        let label_width = (width - 2. * status_width - 2. * spacing.x).max(1.);
        egui::Grid::new(salt)
            .num_columns(3)
            .spacing(spacing)
            .show(ui, |ui| {
                let normal = palette(ui).text;
                for (text, column) in [
                    ("Fresh login test", label_width),
                    ("Expected", status_width),
                    ("Actual", status_width),
                ] {
                    probe_cell(ui, text, column, true, normal);
                }
                ui.end_row();
                for p in probes {
                    probe_cell(ui, &p.label, label_width, false, normal);
                    probe_cell(ui, status(p.expected), status_width, false, normal);
                    let color = if p.accepted == p.expected {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    };
                    probe_cell(ui, status(p.accepted), status_width, false, color);
                    ui.end_row();
                }
            });
    }
}

fn actual(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    controls::form_row(ui, "Sample inputs", |ui, w| {
        let response = controls::dropdown(ui, "password-sample", w)
            .selected_text(&t.cases[usize::from(s.selected_case)].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_label(index == usize::from(s.selected_case), &c.label);
                    record(m, format!("case-{index}"), &r, ui);
                    if r.clicked() {
                        *notice = s.select_case(t, index).err();
                    }
                }
            });
        record(m, "case", &response.response, ui);
    });
    let r = ui.add(button("Run command"));
    record(m, "run", &r, ui);
    if r.clicked() {
        *notice = s.run(t).err();
    }
    ui.strong("Actual command and verification results");
    if let Some(error) = notice {
        lesson_presentation::rich(ui, RichText::new(error.as_str()).color(palette(ui).error));
    }
    let Some(output) = &s.output else {
        let r = lesson_presentation::prose(
            ui,
            "No current results. Run command tests this sample; Check work tests every sample. Each run starts from its original data.",
        );
        m.output = Some(r.rect);
        return;
    };
    let inner = ui.scope(|ui| {
        lesson_presentation::monospace(ui, &output.feedback, true);
        if !output.probes.is_empty() {
            ui.separator();
            probe_table(ui, &output.probes, (&t.id, "actual-probes"));
            lesson_presentation::prose(
                ui,
                if output.directory_preserved {
                    "Directory fields: Unchanged"
                } else {
                    "Directory fields: Changed unexpectedly"
                },
            );
        }
    });
    m.output = Some(inner.response.rect);
}
fn results(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
    practice: bool,
) {
    let r = ui.add(button(if practice {
        "Check example"
    } else {
        "Check work"
    }));
    record(m, "check", &r, ui);
    if r.clicked() {
        *notice = s.check(t).err();
    }
    if !s.checked || s.checked_draft.as_ref() != Some(&s.draft) {
        return;
    }
    let reports = match t.reports(&s.draft) {
        Ok(rows) => rows,
        Err(e) => {
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let passed = reports.iter().filter(|r| r.passed).count();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if passed == reports.len() {
            "Password change verified"
        } else {
            "Password change needs attention"
        })
        .strong()
        .color(if passed == reports.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, reports.len(), r.rect));
    lesson_presentation::prose(
        ui,
        &format!(
            "{passed}/{} samples match. Completion requires the requested replacement, rejection of previous credentials and preservation of unrelated data.",
            reports.len()
        ),
    );
    if let Some(first) = reports.first().filter(|first| {
        !first.passed
            && reports
                .iter()
                .all(|r| !r.passed && r.feedback == first.feedback)
    }) {
        lesson_presentation::prose(ui, &first.feedback);
    } else {
        for row in reports.iter().filter(|r| !r.passed) {
            lesson_presentation::rich(
                ui,
                RichText::new(format!("Needs attention · {}", row.label)).strong(),
            );
            lesson_presentation::prose(ui, &row.feedback);
        }
    }
    controls::collapsing_header(ui, "All sample results and actual login tests").show(ui, |ui| {
        for row in reports {
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "{} · {}",
                    if row.passed {
                        "Pass"
                    } else {
                        "Needs attention"
                    },
                    row.label
                ))
                .strong(),
            );
            lesson_presentation::prose(ui, &row.feedback);
            if !row.probes.is_empty() {
                probe_table(ui, &row.probes, (&t.id, &row.label, "graded-probes"));
            }
        }
    });
}
impl LabApp {
    pub(super) fn ldap_password_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::LdapPassword { definition }, engine::State::LdapPassword { simulation }) =
            (&source.goal, &a.state)
        else {
            return;
        };
        let mut candidate = (**simulation).clone();
        let before = candidate.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            candidate.view
        };
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    record(&mut self.metrics.ldap_password, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                        self.notice = None;
                    }
                }
            });
            candidate.view = view;
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on supplied directory identities and utility commands. Learn connection protection and the separate password prompts before independent work.",
                std::iter::empty(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.ldap_password, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                        self.notice = None;
                    }
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            let task = if practice {
                &definition.example
            } else {
                &definition.initial
            };
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.push_id(("password-workspace",&source.id,practice,challenge),|ui|{ui.add_enabled_ui(active,|ui|{
                self.metrics.ldap_password.side_by_side=ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.;
                if self.metrics.ldap_password.side_by_side{ui.columns(2,|columns|{inputs(&mut columns[0],scope,task,&mut self.metrics.ldap_password,&mut self.notice,(&source,practice),&mut self.focus_input);actual(&mut columns[1],scope,task,&mut self.metrics.ldap_password,&mut self.notice);});}else{inputs(ui,scope,task,&mut self.metrics.ldap_password,&mut self.notice,(&source,practice),&mut self.focus_input);actual(ui,scope,task,&mut self.metrics.ldap_password,&mut self.notice);}
                if practice{lesson_presentation::practice_step(ui,&source,2);}
                results(ui,scope,task,&mut self.metrics.ldap_password,&mut self.notice,practice);
                if practice{
                    let c=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{lesson_presentation::monospace(ui,&task.reference.config,true);lesson_presentation::monospace(ui,&task.reference.command,true);lesson_presentation::prose(ui,"Bind and old target: Lab current password. Replacement: Lab replacement password.");let r=ui.add(button("Fill worked example"));record(&mut self.metrics.ldap_password,"fill-example",&r,ui);if r.clicked(){self.notice=scope.edit(task.reference.clone()).err();}lesson_presentation::prose(ui,"Fills only the separate practice work without running it.");});record(&mut self.metrics.ldap_password,"worked",&c.header_response,ui);
                    let r=ui.add(button("Reset example"));record(&mut self.metrics.ldap_password,"reset-example",&r,ui);if r.clicked(){match work::Scope::new(task){Ok(s)=>{*scope=s;self.notice=None;},Err(e)=>self.notice=Some(e)}}
                }
                let c=controls::collapsing_header(ui,"Help, supported subset and references").show(ui,|ui|{
                    if !practice{let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.ldap_password,"hint",&r,ui);if r.clicked(){a.hints+=1;self.dirty=true;}for h in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,h);}controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});}
                    lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}
                });record(&mut self.metrics.ldap_password,"help",&c.header_response,ui);
            });});
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LdapPassword {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty |= changed;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, i);
        }
    }
}
