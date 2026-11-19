//! Native LDIF editor beside actual in-memory request/state results.
use super::*;
use dlms_learning_lab::ldap_change_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub editor_text_clip: Option<egui::Rect>,
    pub output: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub command: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn editor(
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
    let mut parts = if t.utility.is_some() {
        match work::utility::Draft::parse(&s.draft) {
            Ok(p) => Some(p),
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        }
    } else {
        None
    };
    let before_parts = parts.clone();
    if let (Some(u), Some(p)) = (&t.utility, &mut parts) {
        utility_controls(ui, p, u, &t.id, m, focus);
    }
    ui.strong(
        if t.utility
            .as_ref()
            .is_some_and(work::utility::Utility::is_dn_list)
        {
            "DN retirement list"
        } else if t.utility.is_some() {
            "LDIF virtual input"
        } else {
            "LDIF data editor"
        },
    );
    if let Some(u) = &t.utility {
        lesson_presentation::monospace(ui, &u.input_file, true);
    }
    let mut draft = parts
        .as_ref()
        .map_or_else(|| s.draft.clone(), |p| p.ldif.clone());
    let h = ui.text_style_height(&egui::TextStyle::Monospace).ceil()
        * if t.utility.is_some() { 12. } else { 15. }
        + 2. * f32::from(controls::PAD_Y);
    let width = ui.available_width();
    let mut clip = egui::Rect::NOTHING;
    let (viewport, r) = controls::fixed_viewport(ui, egui::vec2(width, h), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("ldif-draft-scroll")
                .max_height(h)
                .max_width(width)
                .auto_shrink([false, false]),
            |ui| {
                clip = ui.clip_rect();
                ui.add(
                    egui::TextEdit::multiline(&mut draft)
                        .code_editor()
                        .lock_focus(false)
                        .id(egui::Id::new(("ldif-editor", &t.id)))
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .char_limit(work::MAX_DRAFT)
                        .desired_rows(if t.utility.is_some() { 12 } else { 15 })
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                        ),
                )
            },
        )
        .inner
    });
    // The reserved editor viewport includes its native scrollbars. Its parent
    // clip is distinct from the smaller text-content clip inside those bars.
    m.editor = Some((viewport, r.id, ui.clip_rect()));
    m.editor_text_clip = Some(clip);
    record(m, "editor", &r, ui);
    if ui.is_enabled() && *focus {
        r.request_focus();
        *focus = false;
    }
    if r.changed() || parts != before_parts {
        let next = if let Some(mut p) = parts {
            p.ldif = draft;
            p.encode()
        } else {
            Ok(draft)
        };
        *notice = match next {
            Ok(raw) => s.edit(raw).err(),
            Err(e) => {
                s.invalidate();
                Some(e)
            }
        };
    }
}
fn utility_controls(
    ui: &mut egui::Ui,
    draft: &mut work::utility::Draft,
    u: &work::utility::Utility,
    id: &str,
    m: &mut Metrics,
    focus: &mut bool,
) {
    controls::form_row(ui, "Sample credentials", |ui, w| {
        let selected = u
            .credentials
            .iter()
            .find(|c| c.id == draft.credential)
            .map_or("Choose supplied credentials", |c| c.label.as_str());
        let response = controls::dropdown(ui, "ldap-utility-credential", w)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for c in &u.credentials {
                    let r = ui.selectable_value(&mut draft.credential, c.id.clone(), &c.label);
                    record(m, format!("credential-{}", c.id), &r, ui);
                }
            });
        record(m, "credential", &response.response, ui);
    });
    if let Some(c) = u.credentials.iter().find(|c| c.id == draft.credential) {
        lesson_presentation::monospace(ui, &format!("Profile identity: {}", c.principal), true);
    }
    controls::form_row(ui, "Command", |ui, w| {
        let r = ui.add(
            controls::singleline_fitting(ui, &mut draft.command, w)
                .code_editor()
                .lock_focus(false)
                .id(egui::Id::new(("ldap-utility-command", id)))
                .hint_text("ldapadd or ldapmodify with supported options")
                .char_limit(4096),
        );
        m.command = Some((r.rect, r.id, ui.clip_rect()));
        record(m, "command", &r, ui);
        if *focus && ui.is_enabled() {
            r.request_focus();
            *focus = false;
        }
    });
    lesson_presentation::monospace(ui, &format!("Virtual directory: {}", u.uri), true);
}
fn actual(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
    lesson: (&Scenario, bool),
) {
    let (source, practice) = lesson;
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    let mut selected = usize::from(s.selected_case);
    controls::form_row(ui, "Sample inputs", |ui, w| {
        let c = controls::dropdown(ui, "ldif-case", w)
            .selected_text(&t.cases[selected].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, i, &c.label);
                    record(m, format!("case-{i}"), &r, ui);
                }
            });
        record(m, "case", &c.response, ui);
    });
    if selected != usize::from(s.selected_case) {
        *notice = s.select_case(t, selected).err();
    }
    let r = ui.add(button(if t.utility.is_some() {
        "Run command"
    } else {
        "Apply LDIF"
    }));
    record(m, "apply", &r, ui);
    if r.clicked() {
        *notice = s.run(t).err();
    }
    if let Some(e) = notice.as_deref() {
        lesson_presentation::rich(
            ui,
            RichText::new(format!("Not applied · {e}")).color(palette(ui).error),
        );
    }
    ui.strong("Actual request results and directory");
    let Some(output) = &s.output else {
        // A short explanation is ordinary content, not a zero-height scroll
        // viewport. Keep its native text layout, paint clip and hit rectangle.
        let r = lesson_presentation::prose(
            ui,
            if t.utility.is_some() {
                "No current command output. Run command inspects the selected sample; Check work reports every sample separately."
            } else {
                "No current Apply LDIF output. Check work reports each sample separately. Each application starts from the sample's original directory."
            },
        );
        m.output = Some(r.rect);
        return;
    };
    let h = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 15.;
    let out = controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt("ldif-actual-scroll")
            .max_height(h)
            .min_scrolled_height(h)
            .auto_shrink([false, false]),
        |ui| {
            lesson_presentation::monospace(ui, &output.feedback, true);
            ui.separator();
            lesson_presentation::monospace(ui, &output.actual, true);
        },
    );
    m.output = Some(out.inner_rect);
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &work::Task, m: &mut Metrics) {
    if !s.checked || s.checked_draft != s.draft {
        return;
    }
    let reports = match t.reports(&s.draft) {
        Ok(r) => r,
        Err(e) => {
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let passed = reports.iter().filter(|r| r.passed).count();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if passed == reports.len() {
            "All directory requirements match"
        } else {
            "Directory changes need attention"
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
            "{passed}/{} samples match. Every request must succeed and the resulting directory must preserve unrelated data.",
            reports.len()
        ),
    );
    let common = reports.first().filter(|first| {
        !first.passed
            && reports
                .iter()
                .all(|r| !r.passed && r.feedback == first.feedback)
    });
    if let Some(row) = common {
        lesson_presentation::prose(ui, &row.feedback);
    } else {
        for row in reports.iter().filter(|r| !r.passed) {
            lesson_presentation::rich(
                ui,
                RichText::new(format!("Needs attention · {}", row.label)).strong(),
            );
            lesson_presentation::prose(ui, &row.feedback);
        }
    }
    controls::collapsing_header(ui, "All sample results and actual states").show(ui, |ui| {
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
            if !row.actual.is_empty() {
                controls::collapsing_header(ui, &format!("Actual directory · {}", row.label)).show(
                    ui,
                    |ui| {
                        controls::nested_scroll(
                            ui,
                            egui::ScrollArea::both()
                                .id_salt((&t.id, &row.label, "result-output"))
                                .max_height(
                                    ui.text_style_height(&egui::TextStyle::Monospace) * 12.,
                                ),
                            |ui| {
                                lesson_presentation::monospace(ui, &row.actual, true);
                            },
                        );
                    },
                );
            }
        }
    });
}
impl LabApp {
    pub(super) fn ldap_changes_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::LdapChanges { definition }, engine::State::LdapChanges { simulation }) =
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
                    record(&mut self.metrics.ldap_changes, key, &r, ui);
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
                if definition
                    .initial
                    .utility
                    .as_ref()
                    .is_some_and(work::utility::Utility::is_dn_list)
                {
                    "Builds on directory names and utility credentials. Learn to retire specific people by full name while preserving the rest of the directory."
                } else if definition.initial.utility.is_some() {
                    "Builds on directory names, queries and LDIF. Learn how a command selects supplied input and an authenticated training identity."
                } else {
                    "Builds on directory names and queries. Learn LDIF records before applying independent changes."
                },
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
                    record(&mut self.metrics.ldap_changes, key, &r, ui);
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
            ui.push_id(("ldif-workspace",&source.id,practice,challenge),|ui|{ui.add_enabled_ui(active,|ui|{
                self.metrics.ldap_changes.side_by_side=ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.;
                if self.metrics.ldap_changes.side_by_side {ui.columns(2,|columns|{editor(&mut columns[0],scope,task,&mut self.metrics.ldap_changes,&mut self.notice,(&source,practice),&mut self.focus_input);actual(&mut columns[1],scope,task,&mut self.metrics.ldap_changes,&mut self.notice,(&source,practice));});}else{editor(ui,scope,task,&mut self.metrics.ldap_changes,&mut self.notice,(&source,practice),&mut self.focus_input);actual(ui,scope,task,&mut self.metrics.ldap_changes,&mut self.notice,(&source,practice));}
                if practice{lesson_presentation::practice_step(ui,&source,2);}
                let r=ui.add(primary(ui,if practice{"Check example"}else{"Check work"}));record(&mut self.metrics.ldap_changes,"check",&r,ui);if r.clicked(){self.notice=scope.check(task).err();}
                results(ui,scope,task,&mut self.metrics.ldap_changes);
                let c=controls::collapsing_header(ui,"Original sample directory and supplied rights").show(ui,|ui|{lesson_presentation::monospace(ui,&format!("Supplied identity: {}\nDelegated branch: {}\nCreate people: {} · Delete people: {}\nEditable fields: {}",task.grant.principal,task.grant.subtree,if task.grant.add{"Allowed"}else{"Not allowed"},if task.grant.delete{"Allowed"}else{"Not allowed"},task.grant.attributes.iter().map(|a|a.0.as_str()).collect::<Vec<_>>().join(", ")),true);lesson_presentation::prose(ui,if task.utility.is_some(){"Each command checks the selected fictional credentials against its -D identity. Only the delegated identity receives these write permissions; the other supplied profiles are read-only. No real Bind exchange or server ACL is configured."}else{"These are already-established simulator permissions, not an authentication or server ACL configuration."});if let Ok(original)=work::output(&task.cases[usize::from(scope.selected_case)].initial){controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("ldif-before").max_height(ui.text_style_height(&egui::TextStyle::Monospace)*12.),|ui|{lesson_presentation::monospace(ui,&original,true);});}});record(&mut self.metrics.ldap_changes,"original",&c.header_response,ui);
                if practice {let c=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{if task.utility.is_some(){if let Ok(p)=work::utility::Draft::parse(&task.reference){lesson_presentation::prose(ui,&format!("Sample credential profile: {}",p.credential));lesson_presentation::monospace(ui,&p.command,true);lesson_presentation::monospace(ui,&p.ldif,true);}}else{lesson_presentation::monospace(ui,&task.reference,true);}let r=ui.add(button("Fill worked example"));record(&mut self.metrics.ldap_changes,"fill-example",&r,ui);if r.clicked(){self.notice=scope.edit(task.reference.clone()).err();}lesson_presentation::prose(ui,if task.utility.as_ref().is_some_and(work::utility::Utility::is_dn_list){"Fills the practice command, profile and DN list without running them."}else if task.utility.is_some(){"Fills the practice command, profile and LDIF without running them."}else{"Fills the practice editor without applying requests."});});record(&mut self.metrics.ldap_changes,"worked",&c.header_response,ui);let r=ui.add(button("Reset example"));record(&mut self.metrics.ldap_changes,"reset-example",&r,ui);if r.clicked(){match work::Scope::new(task){Ok(s)=>{*scope=s;self.notice=None;},Err(e)=>self.notice=Some(e)}}}
                let c=controls::collapsing_header(ui,if task.utility.is_some(){"Help and supported directory commands"}else{"Help and supported LDIF"}).show(ui,|ui|{if !practice{let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.ldap_changes,"hint",&r,ui);if r.clicked(){a.hints+=1;self.dirty=true;}for h in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,h);}controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});}lesson_presentation::prose(ui,if task.utility.as_ref().is_some_and(work::utility::Utility::is_dn_list){"ldapdelete supports -x -W -H -D, with literal full DNs or -f for the supplied virtual DN list. Enter one complete DN per line, without LDIF headers, comments or blank lines; this subset accepts LF line endings only. The selected fictional profile supplies -W credentials. Requests stop at the first failure; earlier deletions remain in that run. Each new run starts from the original sample. Only individual person entries can be deleted; recursive removal, containers, controls, TLS, SASL and password management are unsupported. No host command, OS file or network connection is used."}else if task.utility.is_some(){"Run command uses only this editor and supplied virtual directory. The selected fictional credentials stand in for the -W prompt; no personal password is requested or saved. ldapadd/ldapmodify support -x -W -H -D and optional -a/-f. No shell, OS files or network connection. A later record error retains earlier successful requests in that run; every new run starts from the original sample. Modify is atomic within one request. Version1 LDIF, restricted Add/Delete and mail/description edits only; implicit default replacement, controls, TLS, SASL and password management are unsupported."}else{work::HELP});lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}});record(&mut self.metrics.ldap_changes,"help",&c.header_response,ui);
            });});
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LdapChanges {
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
