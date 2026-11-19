//! Native directory-client defaults editor and literal simulated search workspace.
use super::*;
use dlms_learning_lab::ldap_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub config: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub command: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub output: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn configuration(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
    source: &Scenario,
    practice: bool,
) {
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    ui.strong("Virtual client defaults");
    let mut config = s.config.clone();
    let h = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 6.
        + 2. * f32::from(controls::PAD_Y);
    let width = ui.available_width();
    let mut clip = egui::Rect::NOTHING;
    let (viewport, r) = controls::fixed_viewport(ui, egui::vec2(width, h), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("ldap-config-scroll")
                .max_height(h)
                .max_width(width)
                .auto_shrink([false, false]),
            |ui| {
                clip = ui.clip_rect();
                ui.add(
                    egui::TextEdit::multiline(&mut config)
                        .code_editor()
                        .lock_focus(false)
                        .id(egui::Id::new(("ldap-config", &t.id)))
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .char_limit(dlms_learning_lab::ldap_client::MAX_CONFIG)
                        .desired_rows(6)
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                        ),
                )
            },
        )
        .inner
    });
    m.config = Some((viewport, r.id, clip));
    if r.changed() {
        *notice = s.edit_config(config).err();
    }
    let r = ui.add(button("Apply configuration"));
    record(m, "apply", &r, ui);
    if r.clicked() {
        *notice = s.apply().err();
    }
    lesson_presentation::prose(
        ui,
        if s.config == s.applied {
            "The displayed defaults are applied to searches."
        } else {
            "Edited · apply these defaults before running a search."
        },
    );
}
fn search(
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
        lesson_presentation::practice_step(ui, source, 1);
    }
    let mut selected = usize::from(s.selected_case);
    controls::form_row(ui, "Sample inputs", |ui, w| {
        let r = controls::dropdown(ui, "ldap-case", w)
            .selected_text(&t.cases[selected].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, i, &c.label);
                    record(m, format!("case-{i}"), &r, ui);
                }
            });
        record(m, "case", &r.response, ui);
    });
    if selected != usize::from(s.selected_case) {
        *notice = s.select_case(t, selected).err();
    }
    ui.strong("Search command");
    let mut command = s.command.clone();
    let width = ui.available_width();
    let r = ui.add(
        controls::singleline_fitting(ui, &mut command, width)
            .font(egui::TextStyle::Monospace)
            .id(egui::Id::new(("ldap-command", &t.id)))
            .char_limit(dlms_learning_lab::ldap_client::MAX_COMMAND)
            .hint_text("ldapsearch -x -LLL 'FILTER' attributes"),
    );
    m.command = Some((r.rect, r.id, ui.clip_rect()));
    record(m, "command", &r, ui);
    if ui.is_enabled() && *focus {
        r.request_focus();
        *focus = false;
    }
    if r.changed() {
        *notice = s.edit_command(command).err();
    }
    // Single-line TextEdit releases focus on Enter; retain that event's owner.
    let enter = ui.is_enabled()
        && (r.has_focus() || r.lost_focus())
        && ui.input(|i| i.key_pressed(egui::Key::Enter));
    let r = ui.add(button("Run search"));
    record(m, "run", &r, ui);
    if r.clicked() || enter {
        *notice = None;
        if let Err(error) = s.run(t) {
            s.output = Some(format!("Search not run: {error}"));
        }
    }
    ui.strong("Actual search result");
    let h = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 10.;
    let out = controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt("ldap-output")
            .max_height(h)
            .min_scrolled_height(h)
            .auto_shrink([false, false]),
        |ui| {
            if let Some(output) = &s.output {
                lesson_presentation::monospace(ui, output, true);
            } else {
                lesson_presentation::prose(
                    ui,
                    "No current Run search output. Check work reports each case below.",
                );
            }
        },
    );
    m.output = Some(out.inner_rect);
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &work::Task, m: &mut Metrics) {
    if !s.checked || s.config != s.checked_config || s.command != s.checked_command {
        return;
    }
    let reports = match t.reports(&s.config, &s.command) {
        Ok(r) => r,
        Err(e) => {
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let passed = reports.iter().filter(|r| r.passed).count();
    let heading = lesson_presentation::rich(
        ui,
        RichText::new(if passed == reports.len() {
            "All directory requirements match"
        } else {
            "Directory work needs attention"
        })
        .strong()
        .color(if passed == reports.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    lesson_presentation::prose(
        ui,
        &format!(
            "{passed}/{} cases match. Each case checks returned entries and attributes, including the configured defaults.",
            reports.len()
        ),
    );
    let common_diagnostic = reports.first().filter(|first| {
        !first.passed
            && !first.ran
            && first.actual.is_empty()
            && reports
                .iter()
                .all(|r| !r.passed && !r.ran && r.actual.is_empty() && r.feedback == first.feedback)
    });
    if let Some(report) = common_diagnostic {
        lesson_presentation::prose(ui, &report.feedback);
    }
    for report in reports
        .iter()
        .filter(|r| !r.passed && common_diagnostic.is_none())
    {
        lesson_presentation::rich(
            ui,
            RichText::new(format!("Needs work: {}", report.label)).strong(),
        );
        lesson_presentation::prose(ui, &report.feedback);
        controls::collapsing_header(ui, "Compare expected and actual entries")
            .id_salt((&t.id, &report.label))
            .show(ui, |ui| {
                ui.strong("Expected result for this case");
                lesson_presentation::monospace(
                    ui,
                    if report.expected.is_empty() {
                        "No matching entries."
                    } else {
                        &report.expected
                    },
                    true,
                );
                ui.strong("Actual result from Check work");
                lesson_presentation::monospace(
                    ui,
                    if !report.ran {
                        "Not run: read the diagnostic above."
                    } else if report.actual.is_empty() {
                        "No matching entries (successful search)."
                    } else {
                        &report.actual
                    },
                    true,
                );
            });
    }
    let c = controls::collapsing_header(ui, "All case results").show(ui, |ui| {
        for r in &reports {
            lesson_presentation::prose(
                ui,
                &format!(
                    "{}: {}",
                    if r.passed { "Matches" } else { "Needs work" },
                    r.label
                ),
            );
        }
    });
    record(m, "check-details", &c.header_response, ui);
    m.result = Some((
        passed,
        reports.len(),
        egui::Rect::from_min_max(
            heading.rect.min,
            egui::pos2(ui.max_rect().right(), c.header_response.rect.bottom()),
        ),
    ));
}
impl LabApp {
    pub(super) fn ldap_client_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::LdapClient { definition }, engine::State::LdapClient { simulation }) =
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
                    record(&mut self.metrics.ldap_client, key, &r, ui);
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
                "Builds on client configuration and Linux command basics. Learn directory names, search scope and filters before changing the client defaults.",
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
                    record(&mut self.metrics.ldap_client, key, &r, ui);
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
            ui.push_id(("ldap-workspace",&source.id,practice,challenge),|ui|{ui.add_enabled_ui(active,|ui|{
    self.metrics.ldap_client.side_by_side=ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.;
    if self.metrics.ldap_client.side_by_side{ui.columns(2,|columns|{configuration(&mut columns[0],scope,task,&mut self.metrics.ldap_client,&mut self.notice,&source,practice);search(&mut columns[1],scope,task,&mut self.metrics.ldap_client,&mut self.notice,(&source,practice),&mut self.focus_input);});}
    else{configuration(ui,scope,task,&mut self.metrics.ldap_client,&mut self.notice,&source,practice);search(ui,scope,task,&mut self.metrics.ldap_client,&mut self.notice,(&source,practice),&mut self.focus_input);}
    if practice{lesson_presentation::practice_step(ui,&source,2);}
    let r=ui.add(primary(ui,if practice{"Check example"}else{"Check work"}));record(&mut self.metrics.ldap_client,"check",&r,ui);if r.clicked(){self.notice=scope.check(task).err();}
    if let Some(error)=self.notice.as_deref(){lesson_presentation::rich(ui,RichText::new(error).color(palette(ui).error));}
    results(ui,scope,task,&mut self.metrics.ldap_client);
    let c=controls::collapsing_header(ui,"Supplied directories and default-setting checks").show(ui,|ui|{let c=&task.cases[usize::from(scope.selected_case)];for d in &c.directories{lesson_presentation::monospace(ui,&format!("{}\nNaming context: {}",d.uri,d.directory.suffix),true);for e in &d.directory.entries{lesson_presentation::monospace(ui,&e.dn,true);}}ui.separator();for c in task.cases.iter().filter(|c|!c.probe.is_empty()){lesson_presentation::prose(ui,&c.label);lesson_presentation::monospace(ui,&c.probe,true);lesson_presentation::prose(ui,&c.reason);}});record(&mut self.metrics.ldap_client,"directories",&c.header_response,ui);
    if practice{let c=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{lesson_presentation::monospace(ui,&task.reference_config,true);lesson_presentation::monospace(ui,&task.reference_query,true);let r=ui.add(button("Fill worked example"));record(&mut self.metrics.ldap_client,"fill-example",&r,ui);if r.clicked(){self.notice=scope.edit_config(task.reference_config.clone()).and_then(|_|scope.edit_command(task.reference_query.clone())).err();}lesson_presentation::prose(ui,"Fills the two practice drafts. Apply configuration and Run search to observe the result.");});record(&mut self.metrics.ldap_client,"worked",&c.header_response,ui);let r=ui.add(button("Reset example"));record(&mut self.metrics.ldap_client,"reset-example",&r,ui);if r.clicked(){match work::Scope::new(task){Ok(s)=>{*scope=s;self.notice=None;},Err(e)=>self.notice=Some(e)}}}
    let c=controls::collapsing_header(ui,"Help and supported search syntax").show(ui,|ui|{if !practice{let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.ldap_client,"hint",&r,ui);if r.clicked(){a.hints+=1;self.dirty=true;}for h in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,h);}controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});}lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}});record(&mut self.metrics.ldap_client,"help",&c.header_response,ui);
   });});
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LdapClient {
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
