//! Native schema editor and filter beside actual supplied consumer results.
use super::*;
use dlms_learning_lab::{ldap_schema as model, ldap_schema_query_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub editor_rows: Option<usize>,
    pub editor_text_clip: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub output: Option<egui::Rect>,
    pub filter: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub case_results: Vec<(bool, bool, usize, usize)>,
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
    let mut draft = s.schema.clone();
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
                    .char_limit(work::MAX_SCHEMA)
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
    if draft != s.schema {
        *notice = s.edit_schema(draft).err();
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
        let response = controls::dropdown(ui, "schema-consumer-case", width)
            .selected_text(&t.cases[usize::from(s.selected_case)].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_label(index == usize::from(s.selected_case), &c.label);
                    record(m, format!("case-{index}"), &r, ui);
                    if r.clicked() {
                        *notice = s.select(t, index).err();
                    }
                }
            });
        record(m, "case", &response.response, ui);
    });
    let c = &t.cases[usize::from(s.selected_case)];
    let supplied = controls::collapsing_header(ui, "Supplied sample entries").show(ui, |ui| {
        lesson_presentation::code_value(
            ui,
            "consumer-supplied",
            &entries_text(&c.entries),
            ui.available_width(),
        );
    });
    record(m, "supplied", &supplied.header_response, ui);
    let mut filter = s.filter.clone();
    let label = ui.strong("LDAP filter");
    let width = ui.available_width();
    let r = ui
        .add(
            controls::singleline_fitting(ui, &mut filter, width)
                .font(egui::TextStyle::Monospace)
                .id_salt(("consumer-filter", &t.id))
                .hint_text("Parenthesized LDAP filter")
                .char_limit(work::MAX_FILTER),
        )
        .labelled_by(label.id);
    m.filter = Some((r.rect, r.id, ui.clip_rect()));
    record(m, "filter", &r, ui);

    if filter != s.filter {
        *notice = s.edit_filter(filter).err();
    }
    lesson_presentation::prose(
        ui,
        "Run the filter against every supplied entry in this sample. Only True selects an entry; False and Undefined do not.",
    );
    let r = ui.add_sized(
        egui::vec2(ui.available_width(), controls::height(ui)),
        button("Apply schema and run filter"),
    );
    record(m, "run", &r, ui);
    if r.clicked() {
        *notice = s.run(t).err();
    }
    let area=ui.scope(|ui|{
  if let Some(error)=notice.as_deref(){
   ui.strong("Input needs attention");
   let response=lesson_presentation::prose(ui,error);record(m,"run-error",&response,ui);
  }
  if let Some(report)=&s.output{
   ui.strong(format!("Actual result: {} of {} entries selected",report.actual.len(),report.observations.len()));
   let actual=if report.actual.is_empty(){"No matching entries".into()}else{report.actual.join("\n")};
   lesson_presentation::code_value(ui,"consumer-selected-actual",&actual,ui.available_width());
   lesson_presentation::prose(ui,if report.passed{"Current sample: selection and comparison rules match the requirements."}else{"Current sample needs attention. A correct selection alone does not prove the comparison rules are correct."});
   if let Some(issue)=report.issues.iter().find(|s|s.contains("Undefined")).or_else(||report.issues.first()){lesson_presentation::prose(ui,issue);}
   let details=controls::collapsing_header(ui,"Actual comparison details").show(ui,|ui|{
    for o in &report.observations{lesson_presentation::monospace(ui,&format!("{:?} · {}",o.truth,o.dn),true);}
    for(passed,label)in &report.rule_checks{lesson_presentation::prose(ui,&format!("{} · {label}",if *passed{"Pass"}else{"Needs attention"}));}
   });record(m,"observations",&details.header_response,ui);
  }else{lesson_presentation::prose(ui,"Actual result: not run for the current schema, filter and sample.");}
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
        "Check configuration"
    }));
    record(m, "check", &r, ui);
    if r.clicked() {
        *notice = s.check(t).err();
    }
    if !s.checked || s.checked_schema != s.schema || s.checked_filter != s.filter {
        return;
    }
    let rows = match t.reports(&s.schema, &s.filter) {
        Ok(v) => v,
        Err(e) => {
            let r = ui.strong("Schema/filter checks could not run");
            m.result = Some((0, t.total_checks(), r.rect));
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let passed = rows.iter().filter(|r| r.passed).count();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if passed == rows.len() {
            "Schema and consumer checks verified"
        } else {
            "Schema or consumer checks need attention"
        })
        .strong()
        .color(if passed == rows.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, rows.len(), r.rect));
    lesson_presentation::prose(
        ui,
        &format!(
            "{passed}/{} checks pass: {} supplied rosters plus a check with different names and IDs. Both selected entries and comparison behavior must match.",
            rows.len(),
            t.cases.len()
        ),
    );
    for row in &rows {
        m.case_results.push((
            row.passed,
            row.selection_matches,
            row.rule_checks.iter().filter(|r| r.0).count(),
            t.rules.len(),
        ));
        ui.strong(format!(
            "{} · {}",
            if row.passed {
                "Pass"
            } else {
                "Needs attention"
            },
            row.label
        ));
        lesson_presentation::prose(
            ui,
            &format!(
                "Selection: {}. Comparison rules: {}/{} pass.",
                if !row.ran {
                    "not run"
                } else if row.selection_matches {
                    "matches"
                } else {
                    "differs"
                },
                row.rule_checks.iter().filter(|r| r.0).count(),
                t.rules.len()
            ),
        );
        if let Some(issue) = row
            .issues
            .iter()
            .find(|s| s.contains("Undefined"))
            .or_else(|| row.issues.first())
        {
            lesson_presentation::prose(ui, issue);
        }
    }
    let all = controls::collapsing_header(ui, "Expected and actual entries · all sample details")
        .show(ui, |ui| {
            for (index, row) in rows.iter().enumerate() {
                ui.strong(&row.label);
                ui.label("Expected entries");
                lesson_presentation::code_value(
                    ui,
                    ("consumer-expected", index),
                    &row.expected.join("\n"),
                    ui.available_width(),
                );
                ui.label("Actual entries");
                lesson_presentation::code_value(
                    ui,
                    ("consumer-actual", index),
                    if row.ran {
                        if row.actual.is_empty() {
                            "No matching entries".into()
                        } else {
                            row.actual.join("\n")
                        }
                    } else {
                        "Not run".into()
                    }
                    .as_str(),
                    ui.available_width(),
                );
                for issue in &row.issues {
                    lesson_presentation::prose(ui, issue);
                }
            }
        });
    record(m, "all-results", &all.header_response, ui);
}
impl LabApp {
    pub(super) fn ldap_schema_query_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (
            engine::Goal::LdapSchemaQuery { definition },
            engine::State::LdapSchemaQuery { simulation },
        ) = (&source.goal, &a.state)
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
                    record(&mut self.metrics.ldap_schema_query, key, &r, ui);
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
                "Builds on the custom-schema lesson and parenthesized filters. Here, verify how a schema changes which supplied entries a consumer selects.",
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
                    record(&mut self.metrics.ldap_schema_query, key, &r, ui);
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
    self.metrics.ldap_schema_query.side_by_side=ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.;
    if self.metrics.ldap_schema_query.side_by_side{ui.columns(2,|cols|{editor(&mut cols[0],scope,task,(&source,practice),&mut self.metrics.ldap_schema_query,&mut self.notice,&mut self.focus_input);trial(&mut cols[1],scope,task,(&source,practice),&mut self.metrics.ldap_schema_query,&mut self.notice);});}else{editor(ui,scope,task,(&source,practice),&mut self.metrics.ldap_schema_query,&mut self.notice,&mut self.focus_input);ui.add_space(controls::GAP);trial(ui,scope,task,(&source,practice),&mut self.metrics.ldap_schema_query,&mut self.notice);}
    if practice{lesson_presentation::practice_step(ui,&source,2);}results(ui,scope,task,practice,&mut self.metrics.ldap_schema_query,&mut self.notice);
    if practice{
     let c=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{lesson_presentation::code_value(ui,"consumer-worked-schema",&task.reference_schema,ui.available_width());lesson_presentation::code_value(ui,"consumer-worked-filter",&task.reference_filter,ui.available_width());let r=ui.add(button("Fill worked example"));record(&mut self.metrics.ldap_schema_query,"fill-example",&r,ui);if r.clicked(){self.notice=scope.edit_schema(task.reference_schema.clone()).and_then(|_|scope.edit_filter(task.reference_filter.clone())).err();}lesson_presentation::prose(ui,"Fills the workshop schema and filter without running or grading them.");});record(&mut self.metrics.ldap_schema_query,"worked",&c.header_response,ui);
     let r=ui.add(button("Reset example"));record(&mut self.metrics.ldap_schema_query,"reset-example",&r,ui);if r.clicked(){*scope=work::Scope::new(task);self.notice=None;}
    }
    let c=controls::collapsing_header(ui,"Help, supported subset and references").show(ui,|ui|{
     if !practice{let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.ldap_schema_query,"hint",&r,ui);if r.clicked(){a.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});}
     lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}
    });record(&mut self.metrics.ldap_schema_query,"help",&c.header_response,ui);
   });});
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LdapSchemaQuery {
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
