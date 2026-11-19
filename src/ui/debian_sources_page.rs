//! Native source-selection editor and actual cached-metadata evidence.
use super::*;
use dlms_learning_lab::{debian_source_cache as cache, debian_sources_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    pub query: Option<(egui::Rect, egui::Id)>,
    pub results: Vec<bool>,
    pub observation: Option<egui::Rect>,
    pub report: Option<egui::Rect>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn editor(
    ui: &mut egui::Ui,
    source: &Scenario,
    d: &work::Definition,
    scope: &mut work::Scope,
    practice: bool,
    active: bool,
    m: &mut Metrics,
) -> Option<String> {
    let task = if practice { &d.example } else { &d.initial };
    let mut notice = None;
    let feedback_key = egui::Id::new(("sources-editor-feedback", &source.id, practice));
    let original_draft = scope.draft.clone();
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    lesson_presentation::rich(
        ui,
        RichText::new("Source editor · virtual deb822 selection excerpt").strong(),
    );
    if practice {
        let r = ui.add_enabled(active, button("Fill worked source excerpt"));
        record(m, "fill-worked", &r, ui);
        if (r.clicked())
            && let Err(e) = scope.edit(&d.example_worked.sources)
        {
            notice = Some(e);
        }
    }
    let mut draft = scope.draft.clone();
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 9. + 2. * f32::from(controls::PAD_Y);
    let width = ui.available_width();
    let (viewport, r) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt(("apt-sources-editor-scroll", &source.id, practice))
                .auto_shrink([false, false])
                .max_height(height)
                .max_width(width),
            |ui| {
                ui.add_enabled(
                    active,
                    egui::TextEdit::multiline(&mut draft)
                        .code_editor()
                        .id(egui::Id::new(("apt-sources-editor", &source.id, practice)))
                        .desired_width(ui.available_width())
                        .desired_rows(8)
                        .char_limit(8192),
                )
            },
        )
        .inner
    });
    name_control(&r, "Simulated package source selection");
    // Keep the bounded editor's full layout extent for native reveal/focus.
    // Clipping it to the OUTER window makes its reported bottom stay pinned
    // outside the viewport and reduces every wheel request to a few pixels.
    m.editor = Some((r.rect.intersect(viewport), r.id));
    if (r.changed())
        && let Err(e) = scope.edit(&draft)
    {
        notice = Some(e);
    }
    let shortcut = active
        && r.has_focus()
        && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter));
    if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        r.surrender_focus();
    }
    ui.horizontal_wrapped(|ui|{
  let r=ui.add_enabled(active,button("Save sources"));record(m,"save",&r,ui);
  if r.clicked()||shortcut{notice=Some(match scope.save(task){Ok(())=>"Saved source selection. Cached metadata is unchanged; refresh explicitly when needed.".into(),Err(e)=>e});}
  let r=ui.add_enabled(active,button("Refresh simulated indexes"));record(m,"refresh",&r,ui);
  if r.clicked(){notice=Some(match scope.refresh(task){Err(e)=>e,Ok(rows)=>{
   let updated=rows.iter().filter(|r|r.status==cache::RefreshStatus::Updated).count();let missing=rows.len()-updated;
   if missing==0{format!("Refreshed {updated} selected indexes. No software installed.")}else{format!("Partly refreshed: {updated} updated; {missing} unavailable. Older matching caches are retained; inspect each source below.")}
  }});}
 });
    lesson_presentation::prose(
        ui,
        if scope.draft == scope.state.saved {
            "Editor matches saved selection."
        } else {
            "Unsaved source edits · Save sources before refreshing or checking."
        },
    );
    if scope.draft != original_draft {
        ui.ctx().data_mut(|d| d.remove::<String>(feedback_key));
    }
    if let Some(message) = &notice {
        ui.ctx()
            .data_mut(|d| d.insert_temp(feedback_key, message.clone()));
    }
    if let Some(message) = ui.ctx().data(|d| d.get_temp::<String>(feedback_key)) {
        lesson_presentation::prose(ui, &message);
    }
    notice
}
fn evidence(
    ui: &mut egui::Ui,
    source: &Scenario,
    d: &work::Definition,
    scope: &mut work::Scope,
    practice: bool,
    active: bool,
    m: &mut Metrics,
) -> Option<String> {
    let task = if practice { &d.example } else { &d.initial };
    let mut notice = None;
    let feedback_key = egui::Id::new(("sources-query-feedback", &source.id, practice));
    let original_query = scope.query.clone();
    let mut looked_up = false;
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    lesson_presentation::rich(
        ui,
        RichText::new("Actual cached indexes · saved selection").strong(),
    );
    let indexes = scope
        .state
        .indexes(&task.model)
        .expect("context-bound source state");
    if indexes.is_empty() {
        lesson_presentation::prose(
            ui,
            "No active source targets. Retained cache bytes alone are not available metadata.",
        );
    }
    for index in &indexes {
        ui.horizontal_wrapped(|ui| {
            lesson_presentation::rich(
                ui,
                RichText::new(format!("{}/{}", index.key.suite, index.key.component)).strong(),
            );
            let label = match index.status {
                cache::IndexStatus::Current => "Current",
                cache::IndexStatus::Retained => "Retained matching cache",
                cache::IndexStatus::Missing => "Missing cache",
            };
            lesson_presentation::prose(ui, label);
        });
        lesson_presentation::code_value(
            ui,
            ("sources-active-uri", &source.id, practice, &index.key),
            &index.key.uri,
            ui.available_width(),
        );
        if let Some(snapshot) = &index.snapshot {
            lesson_presentation::prose(ui, &format!("Snapshot · {snapshot}"));
        }
        ui.add_space(4.);
    }
    controls::collapsing_header(ui, "Supplied repository targets")
        .id_salt(("sources-supplied", &source.id, practice))
        .show(ui, |ui| {
            for target in &task.model.targets {
                lesson_presentation::code_value(
                    ui,
                    ("sources-supplied-target", &source.id, practice, &target.key),
                    &format!(
                        "{} {} {}",
                        target.key.uri, target.key.suite, target.key.component
                    ),
                    ui.available_width(),
                );
                lesson_presentation::prose(
                    ui,
                    if target.published.is_some() {
                        "Supplied publication available for refresh"
                    } else {
                        "Publication unavailable; an older matching cache can still be read"
                    },
                );
            }
        });
    ui.separator();
    if practice {
        lesson_presentation::practice_step(ui, source, 2);
    }
    let mut query = scope.query.clone();
    ui.horizontal_wrapped(|ui| {
        let label = ui.label("Package name");
        let field = controls::singleline(ui, &mut query)
            .hint_text("Exact package name")
            .desired_width(
                (ui.available_width()
                    - controls::button_width(ui, "Look up package")
                    - ui.spacing().item_spacing.x)
                    .max(ui.text_style_height(&egui::TextStyle::Body) * 8.),
            )
            .char_limit(64);
        let r = ui.add_enabled(active, field).labelled_by(label.id);
        name_control(&r, "Package metadata query");
        m.query = Some((r.rect, r.id));
        if (r.changed())
            && let Err(e) = scope.set_query(&query)
        {
            notice = Some(e);
        }
        let enter = active
            && (r.has_focus() || r.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let r = ui.add_enabled(active, button("Look up package"));
        record(m, "lookup", &r, ui);
        if r.clicked() || enter {
            looked_up = true;
            if let Err(e) = scope.lookup(task) {
                notice = Some(e);
            }
        }
    });
    if looked_up || scope.query != original_query {
        ui.ctx().data_mut(|d| d.remove::<String>(feedback_key));
    }
    if let Some(message) = &notice {
        ui.ctx()
            .data_mut(|d| d.insert_temp(feedback_key, message.clone()));
    }
    if let Some(message) = ui.ctx().data(|d| d.get_temp::<String>(feedback_key)) {
        lesson_presentation::prose(ui, &message);
    }
    if let Some(name) = &scope.observed {
        let area=ui.scope(|ui| {
        let rows = scope
            .state
            .query(&task.model, name)
            .expect("validated observed query");
        lesson_presentation::rich(
            ui,
            RichText::new(format!("Actual lookup · {name}")).strong(),
        );
        if rows.is_empty() {
            lesson_presentation::prose(
                ui,
                "No matching package rows in the selected cached indexes. Check source selection and cache status; a lookup does not refresh.",
            );
        }
        for (n, row) in rows.iter().enumerate() {
            lesson_presentation::code_value(
                ui,
                ("sources-result", &source.id, practice, n),
                &format!(
                    "{} {} {}",
                    row.row.name,
                    row.row.version.as_str(),
                    match row.row.architecture {
                        dlms_learning_lab::debian_packages::Architecture::Amd64 => "amd64",
                        dlms_learning_lab::debian_packages::Architecture::Arm64 => "arm64",
                        dlms_learning_lab::debian_packages::Architecture::All => "all",
                    }
                ),
                ui.available_width(),
            );
            lesson_presentation::prose(
                ui,
                &format!(
                    "From {}/{} · snapshot {}",
                    row.key.suite, row.key.component, row.snapshot
                ),
            );
        }
        });
        m.observation = Some(area.response.rect);
    } else {
        lesson_presentation::prose(
            ui,
            "Not looked up · enter a package name and select Look up package.",
        );
    }
    notice
}
impl LabApp {
    pub(super) fn debian_sources_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let before_reset = phase.1.then(|| {
            serde_json::to_value(&self.progress.attempts[&self.scenarios[i].id].state).unwrap()
        });
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            if phase.1
                && !self.reset_pending
                && before_reset.as_ref()
                    != Some(
                        &serde_json::to_value(&self.progress.attempts[&self.scenarios[i].id].state)
                            .unwrap(),
                    )
            {
                for practice in [false, true] {
                    for zone in ["sources-editor-feedback", "sources-query-feedback"] {
                        let key = egui::Id::new((zone, &self.scenarios[i].id, practice));
                        ui.ctx().data_mut(|d| d.remove::<String>(key));
                    }
                }
            }
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (
            engine::Goal::DebianSources { definition },
            engine::State::DebianSources { simulation },
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
        let view = if challenge {
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
                    let r = ui.add_enabled(active, button(label).selected(v == view));
                    record(&mut self.metrics.debian_sources, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
        }
        let mut feedback_event = false;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on Debian package administration and repository Contents investigation. This is a selection excerpt; real authentication and trust configuration are outside this editor.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::code_value(
                ui,
                ("sources-worked", &source.id),
                &definition.example_worked.sources,
                ui.available_width(),
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.debian_sources, key, &r, ui);
                    if r.clicked() {
                        candidate.view = v;
                        self.lesson_scroll_reset = true;
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
                lesson_presentation::lead(
                    ui,
                    "PRACTICE TASK",
                    "Edit sources, then inspect their metadata.",
                );
                lesson_presentation::prose(ui, &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let mut message = None;
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 55. {
                ui.columns(2, |columns| {
                    message = editor(
                        &mut columns[0],
                        &source,
                        definition,
                        scope,
                        practice,
                        active,
                        &mut self.metrics.debian_sources,
                    );
                    if let Some(e) = evidence(
                        &mut columns[1],
                        &source,
                        definition,
                        scope,
                        practice,
                        active,
                        &mut self.metrics.debian_sources,
                    ) {
                        message = Some(e);
                    }
                });
            } else {
                message = editor(
                    ui,
                    &source,
                    definition,
                    scope,
                    practice,
                    active,
                    &mut self.metrics.debian_sources,
                );
                ui.separator();
                if let Some(e) = evidence(
                    ui,
                    &source,
                    definition,
                    scope,
                    practice,
                    active,
                    &mut self.metrics.debian_sources,
                ) {
                    message = Some(e);
                }
            }
            if let Some(e) = message {
                self.notice = Some(e);
                feedback_event = true;
            }
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check work"
                    },
                ),
            );
            record(&mut self.metrics.debian_sources, "check", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                scope.check(task);
                self.notice = None;
                feedback_event = true;
            }
            if scope.checked {
                let rows = task.checks(scope);
                let passed = rows.iter().filter(|r| r.0).count();
                self.metrics.debian_sources.results = rows.iter().map(|r| r.0).collect();
                let r = lesson_presentation::rich(
                    ui,
                    RichText::new(format!(
                        "{} · {passed}/{} requirements met",
                        if passed == rows.len() {
                            "Configuration meets the task"
                        } else {
                            "Configuration needs work"
                        },
                        rows.len()
                    ))
                    .strong(),
                );
                let mut report = r.rect;
                if passed == rows.len() {
                    report = report.union(lesson_presentation::prose(ui, &source.explanation).rect);
                } else {
                    let mut first = true;
                    for (_, text) in rows.iter().filter(|r| !r.0) {
                        let r = lesson_presentation::prose(ui, text);
                        if first {
                            report = report.union(r.rect);
                            first = false;
                        }
                    }
                }
                // The native evidence frame must include why an outcome failed,
                // rather than stopping with only its summary on the bottom edge.
                self.metrics.debian_sources.report = Some(report);
                controls::collapsing_header(ui, "All requirement results")
                    .id_salt(("sources-all-results", &source.id, practice))
                    .show(ui, |ui| {
                        for (ok, text) in &rows {
                            lesson_presentation::prose(
                                ui,
                                &format!("{} · {text}", if *ok { "Met" } else { "Needs work" }),
                            );
                        }
                    });
            }
            if practice {
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.debian_sources, "reset-example", &r, ui);
                if r.clicked() {
                    candidate.practice = work::Scope::new(&definition.example);
                    self.notice = None;
                }
            }
            controls::collapsing_header(ui, "Help and simulation scope")
                .id_salt(("sources-help", &source.id, practice, challenge))
                .show(ui, |ui| {
                    lesson_presentation::prose(ui, work::HELP);
                    if !practice {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        let r = ui.add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        );
                        record(&mut self.metrics.debian_sources, "hint", &r, ui);
                        if r.clicked() {
                            a.hints += 1;
                            self.dirty = true;
                        }
                        for hint in source.hints.iter().take(a.hints) {
                            lesson_presentation::prose(ui, hint);
                        }
                        controls::collapsing_header(ui, "Worked approach · reveals a solution")
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, &source.explanation);
                                lesson_presentation::code_value(
                                    ui,
                                    ("sources-assignment-reference", &source.id),
                                    &definition.assignment_reference.sources,
                                    ui.available_width(),
                                );
                            });
                    }
                    controls::collapsing_header(ui, "Technical references").show(ui, |ui| {
                        for url in &source.sources {
                            ui.hyperlink_to(url, url);
                        }
                    });
                });
        }
        if !feedback_event
            && (candidate.assignment.draft != before.assignment.draft
                || candidate.practice.draft != before.practice.draft
                || candidate.assignment.query != before.assignment.query
                || candidate.practice.query != before.practice.query)
        {
            self.notice = None;
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::DebianSources {
                            simulation: Box::new(candidate),
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
