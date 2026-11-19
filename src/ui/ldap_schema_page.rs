//! Native schema editor beside actual sample requests and stored data.
use super::*;
use dlms_learning_lab::{ldap_schema as model, ldap_schema_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub editor_rows: Option<usize>,
    pub editor_text_clip: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub output: Option<egui::Rect>,
    pub preview: Option<(egui::Rect, usize)>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn entries_text(rows: &[model::EntrySource]) -> String {
    let mut lines = Vec::new();
    for entry in rows {
        lines.push(format!("dn: {}", entry.dn));
        for class in &entry.classes {
            lines.push(format!("objectClass: {class}"));
        }
        for field in &entry.fields {
            for value in &field.values {
                lines.push(format!("{}: {value}", field.name));
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}
fn editor(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    lesson: (&Scenario, bool),
    m: &mut Metrics,
    notice: &mut Option<String>,
    focus: &mut bool,
) {
    let (source, practice) = lesson;
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    let label = ui.strong("Schema LDIF editor");
    lesson_presentation::prose(
        ui,
        &format!(
            "Add cn={},cn=schema,cn=config using new OIDs below {}. The lab provides schema-editing rights.",
            t.schema_name, t.oid_branch
        ),
    );
    let width = ui.available_width();
    let height = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 18.
        + 2. * f32::from(controls::PAD_Y);
    let mut draft = s.draft.clone();
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let (viewport, response) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("schema-editor-scroll")
                .max_width(width)
                .min_scrolled_width(width)
                .max_height(height)
                .auto_shrink([false, false]),
            |ui| {
                m.editor_text_clip = Some(ui.clip_rect());
                let out = egui::TextEdit::multiline(&mut draft)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_rows(18)
                    .desired_width(
                        (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                    )
                    .id(egui::Id::new(("schema-document", &t.id)))
                    .hint_text("version: 1")
                    .char_limit(work::MAX_DRAFT)
                    .show(ui);
                m.editor_rows = Some(out.galley.rows.len());
                out.response.labelled_by(label.id)
            },
        )
        .inner
    });
    m.editor = Some((viewport, response.id, ui.clip_rect()));
    m.controls.push(("editor".into(), viewport, ui.clip_rect()));
    if *focus && ui.is_enabled() {
        response.request_focus();
        *focus = false;
    }
    if draft != s.draft {
        *notice = s.edit(draft).err();
    }
}
fn trial(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    lesson: (&Scenario, bool),
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    if lesson.1 {
        lesson_presentation::practice_step(ui, lesson.0, 1);
    }
    controls::form_row(ui, "Sample data", |ui, width| {
        let response = controls::dropdown(ui, "schema-case", width)
            .selected_text(&t.cases[usize::from(s.selected_case)].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_label(index == usize::from(s.selected_case), &c.label);
                    record(m, format!("case-{index}"), &r, ui);
                    if r.clicked() {
                        *notice = s.select(t, index, 0).err();
                    }
                }
            });
        record(m, "case", &response.response, ui);
    });
    let case = &t.cases[usize::from(s.selected_case)];
    controls::form_row(ui, "Entry request", |ui, width| {
        let response = controls::dropdown(ui, "schema-probe", width)
            .selected_text(&case.probes[usize::from(s.selected_probe)].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, p) in case.probes.iter().enumerate() {
                    let r = ui.selectable_label(index == usize::from(s.selected_probe), &p.label);
                    record(m, format!("probe-{index}"), &r, ui);
                    if r.clicked() {
                        *notice = s.select(t, usize::from(s.selected_case), index).err();
                    }
                }
            });
        record(m, "probe", &response.response, ui);
    });
    if !case.initial.is_empty() {
        let original = controls::collapsing_header(ui, "Existing sample entries").show(ui, |ui| {
            lesson_presentation::code_value(
                ui,
                "schema-supplied-entries",
                &entries_text(&case.initial),
                ui.available_width(),
            );
        });
        record(m, "initial-entries", &original.header_response, ui);
    }
    let p = &case.probes[usize::from(s.selected_probe)];
    ui.strong(format!(
        "{} · required result: {}",
        match p.operation {
            work::Operation::Add => "Add entry",
            work::Operation::Replace => "Replace entry fields",
        },
        if p.accept {
            "Accept"
        } else {
            "Reject without changes"
        }
    ));
    lesson_presentation::prose(ui, &p.reason);
    let preview = entries_text(std::slice::from_ref(&p.entry));
    let lines = preview.split('\n').count().clamp(4, 12);
    let height = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * lines as f32;
    let scroll = ui.spacing().scroll;
    let reserve = scroll.bar_width + scroll.bar_inner_margin + scroll.bar_outer_margin;
    let width = ui.available_width();
    let area = controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt("schema-request-preview")
            .max_width(width)
            .min_scrolled_width(width)
            .min_scrolled_height(height)
            .max_height(height + reserve)
            .auto_shrink([false, false]),
        |ui| {
            lesson_presentation::monospace(ui, &preview, false);
        },
    );
    m.preview = Some((area.inner_rect, lines));
    let action = ui.add(button("Apply schema and try request"));
    record(m, "run", &action, ui);
    if action.clicked() {
        *notice = s.run(t).err();
    }
    let area = ui.scope(|ui| {
        if let Some(error) = notice.as_ref() {
            lesson_presentation::rich(ui, RichText::new(error).color(palette(ui).error));
        }
        if let Some(run) = &s.output {
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "Actual result: {} · {}",
                    if run.accepted { "Accepted" } else { "Rejected" },
                    if run.passed {
                        "Matches requirement"
                    } else {
                        "Needs attention"
                    }
                ))
                .strong()
                .color(if run.passed {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            lesson_presentation::prose(ui, &run.feedback);
            for module in &run.modules {
                lesson_presentation::monospace(ui, &format!("Loaded schema: {}", module.dn), true);
            }
            let stored = controls::collapsing_header(
                ui,
                "Actual stored entries · before and after",
            )
            .show(ui, |ui| {
                ui.strong("Before request");
                lesson_presentation::code_value(
                    ui,
                    "schema-before",
                    &entries_text(&run.before),
                    ui.available_width(),
                );
                ui.strong("After request");
                lesson_presentation::code_value(
                    ui,
                    "schema-after",
                    &entries_text(&run.after),
                    ui.available_width(),
                );
            });
            record(m, "stored-entries", &stored.header_response, ui);
        } else {
            lesson_presentation::prose(
                ui,
                "Actual result: not run for the current draft and selected request.",
            );
        }
    });
    m.output = Some(area.response.rect);
}
fn results(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let r = ui.add(button(if practice {
        "Check example"
    } else {
        "Check schema"
    }));
    record(m, "check", &r, ui);
    if r.clicked() {
        *notice = s.check(t).err();
    }
    if !s.checked || s.checked_draft.as_deref() != Some(&s.draft) {
        return;
    }
    let rows = match t.reports(&s.draft) {
        Ok(v) => v,
        Err(e) => {
            let r = ui.strong("Schema not verified");
            m.result = Some((0, t.cases.len(), r.rect));
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let matched = rows.iter().filter(|r| r.passed).count();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if matched == rows.len() {
            "Schema and entry checks verified"
        } else {
            "Schema or entry checks need attention"
        })
        .strong()
        .color(if matched == rows.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((matched, rows.len(), r.rect));
    lesson_presentation::prose(
        ui,
        &format!(
            "{matched}/{} sample sets match. Every valid entry must work, and invalid data must be rejected without changing stored entries.",
            rows.len()
        ),
    );
    // Name actual failing requests; passing cases retain their own correct status.
    for row in rows.iter().filter(|r| !r.passed) {
        let passing = row.runs.iter().filter(|r| r.passed).count();
        ui.strong(format!(
            "Needs attention · {} · {passing}/{} requests match",
            row.label,
            row.runs.len()
        ));
        let names = row
            .runs
            .iter()
            .filter(|r| !r.passed)
            .map(|r| r.label.as_str())
            .collect::<Vec<_>>()
            .join("; ");
        lesson_presentation::prose(ui, &format!("Failing requests: {names}"));
    }
    if let Some(first) = rows.iter().flat_map(|r| &r.runs).find(|r| !r.passed) {
        ui.strong(format!("First issue · {}", first.label));
        lesson_presentation::prose(ui, &first.feedback);
    }
    let all = controls::collapsing_header(ui, "All entry-request results").show(ui, |ui| {
        for (index, row) in rows.iter().enumerate() {
            let r = ui.strong(format!(
                "{} · {}",
                if row.passed {
                    "Pass"
                } else {
                    "Needs attention"
                },
                row.label
            ));
            record(m, format!("sample-result-{index}"), &r, ui);
            for run in &row.runs {
                if !run.passed {
                    lesson_presentation::prose(ui, &run.feedback);
                }
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} · {} · Required {}; actual {}",
                        if run.passed {
                            "Pass"
                        } else {
                            "Needs attention"
                        },
                        run.label,
                        if run.expected_accept {
                            "Accept"
                        } else {
                            "Reject"
                        },
                        if run.accepted { "Accepted" } else { "Rejected" }
                    ),
                );
            }
        }
    });
    record(m, "all-results", &all.header_response, ui);
}
impl LabApp {
    pub(super) fn ldap_schema_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::LdapSchema { definition }, engine::State::LdapSchema { simulation }) =
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
                    record(&mut self.metrics.ldap_schema, key, &r, ui);
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
                "Builds on directory names, entry fields and schema/configuration Add records. Learn types, required fields and inheritance before extending the directory.",
                std::iter::empty(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            if let Some(entry) = lesson_presentation::entry(&source) {
                for (n, row) in entry.worked.iter().enumerate() {
                    ui.strong(&row.label);
                    lesson_presentation::code_value(
                        ui,
                        ("schema-worked", &source.id, n),
                        &row.value,
                        ui.available_width(),
                    );
                    lesson_presentation::prose(ui, &row.explanation);
                    ui.add_space(ui.spacing().item_spacing.y);
                }
            }
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.ldap_schema, key, &r, ui);
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
            ui.push_id(("schema-workspace",&source.id,practice,challenge),|ui|{ui.add_enabled_ui(active,|ui|{
    self.metrics.ldap_schema.side_by_side=ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.;
    if self.metrics.ldap_schema.side_by_side{ui.columns(2,|cols|{editor(&mut cols[0],scope,task,(&source,practice),&mut self.metrics.ldap_schema,&mut self.notice,&mut self.focus_input);trial(&mut cols[1],scope,task,(&source,practice),&mut self.metrics.ldap_schema,&mut self.notice);});}else{editor(ui,scope,task,(&source,practice),&mut self.metrics.ldap_schema,&mut self.notice,&mut self.focus_input);ui.add_space(controls::GAP);trial(ui,scope,task,(&source,practice),&mut self.metrics.ldap_schema,&mut self.notice);}
    if practice{lesson_presentation::practice_step(ui,&source,2);}results(ui,scope,task,practice,&mut self.metrics.ldap_schema,&mut self.notice);
    if practice{
     let c=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{lesson_presentation::monospace(ui,&task.reference,true);let r=ui.add(button("Fill worked example"));record(&mut self.metrics.ldap_schema,"fill-example",&r,ui);if r.clicked(){self.notice=scope.edit(task.reference.clone()).err();}lesson_presentation::prose(ui,"Fills this practice draft without applying the schema or running a request.");});record(&mut self.metrics.ldap_schema,"worked",&c.header_response,ui);
     let r=ui.add(button("Reset example"));record(&mut self.metrics.ldap_schema,"reset-example",&r,ui);if r.clicked(){*scope=work::Scope::new(task);self.notice=None;}
    }
    let c=controls::collapsing_header(ui,"Help, supported subset and references").show(ui,|ui|{
     if !practice{let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.ldap_schema,"hint",&r,ui);if r.clicked(){a.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});}
     lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}
    });record(&mut self.metrics.ldap_schema,"help",&c.header_response,ui);
   });});
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LdapSchema {
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
