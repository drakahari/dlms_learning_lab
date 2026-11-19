//! Focused static write-policy editing with actual bounded directory changes.
use super::*;
use dlms_learning_lab::{ldap_access_workspace as work, ldap_query::Dn};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub editor_rows: Option<usize>,
    pub editor_text_clip: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub output: Option<egui::Rect>,
    pub configuration_inventory: Option<egui::Rect>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
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
    let deploying = t.configuration.is_some();
    let label = ui.strong(if deploying {
        "LDIF configuration change"
    } else {
        "Write access policy"
    });
    lesson_presentation::prose(
        ui,
        if deploying {
            "The lab provides configuration-editing rights. Use version: 1, the configuration entry's full dn, changetype: modify and replace: olcAccess. Values begin with to, followed by by clauses."
        } else {
            "Static access to rules for established callers. Use lowercase keywords and indented by lines, or keep a whole rule on one line."
        },
    );
    let width = ui.available_width();
    let h = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 10.
        + 2. * f32::from(controls::PAD_Y);
    let mut draft = s.draft.clone();
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let (viewport, response) = controls::fixed_viewport(ui, egui::vec2(width, h), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("acl-policy-scroll")
                .max_width(width)
                .min_scrolled_width(width)
                .max_height(h)
                .auto_shrink([false, false]),
            |ui| {
                m.editor_text_clip = Some(ui.clip_rect());
                let out = egui::TextEdit::multiline(&mut draft)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_rows(10)
                    .desired_width(
                        (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                    )
                    .id(egui::Id::new(("acl-policy", &t.id)))
                    .hint_text(if deploying {
                        "version: 1"
                    } else {
                        "access to ..."
                    })
                    .char_limit(work::MAX_DRAFT)
                    .show(ui);
                m.editor_rows = Some(out.galley.rows.len());
                out.response.labelled_by(label.id)
            },
        )
        .inner
    });
    // Record the reserved viewport, including native scrollbars, rather than
    // the deliberately wider document's content rectangle.
    m.editor = Some((viewport, response.id, ui.clip_rect()));
    m.controls.push(("editor".into(), viewport, ui.clip_rect()));
    if *focus && ui.is_enabled() {
        response.request_focus();
        *focus = false;
    }
    if draft != s.draft {
        *notice = s.edit(draft).err();
    }
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
}
fn configuration_inventory(ui: &mut egui::Ui, deployment: &work::Deployment) {
    ui.strong("Supplied configuration entries");
    lesson_presentation::prose(
        ui,
        "Find the data branch in olcSuffix, then use that entry's complete name. These are simulated directory settings, not files to edit on your computer.",
    );
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 54.;
    let width = (ui.available_width() - ui.spacing().item_spacing.x) / 2.;
    if wide {
        egui::Grid::new("config-database-inventory")
            .num_columns(2)
            .show(ui, |ui| {
                ui.strong("Configuration entry");
                ui.strong("Data branch / role");
                ui.end_row();
                for (n, d) in deployment.initial.databases.iter().enumerate() {
                    lesson_presentation::code_value(ui, ("config-name", n), &d.name, width);
                    lesson_presentation::code_value(
                        ui,
                        ("config-suffix", n),
                        d.suffix
                            .as_deref()
                            .unwrap_or(if dlms_learning_lab::ldap_configuration::Name::parse(&d.name).is_ok_and(|n| matches!(n, dlms_learning_lab::ldap_configuration::Name::Database { backend: dlms_learning_lab::ldap_configuration::Backend::Frontend, .. })) {
                                "Inherited rules"
                            } else {
                                "cn=config · protected"
                            }),
                        width,
                    );
                    ui.end_row();
                }
            });
    } else {
        for (n, d) in deployment.initial.databases.iter().enumerate() {
            lesson_presentation::code_value(ui, ("config-name", n), &d.name, ui.available_width());
            lesson_presentation::prose(
                ui,
                &format!(
                    "{}: {}",
                    if d.suffix.is_some() {
                        "olcSuffix"
                    } else {
                        "Role"
                    },
                    d.suffix.as_deref().unwrap_or(
                        if dlms_learning_lab::ldap_configuration::Name::parse(&d.name).is_ok_and(
                            |n| matches!(
                                n,
                                dlms_learning_lab::ldap_configuration::Name::Database {
                                    backend:
                                        dlms_learning_lab::ldap_configuration::Backend::Frontend,
                                    ..
                                }
                            )
                        ) {
                            "Inherited rules"
                        } else {
                            "Configuration · protected"
                        }
                    )
                ),
            );
        }
    }
    controls::collapsing_header(ui, "Inspect supplied access values").show(ui, |ui| {
        match dlms_learning_lab::ldap_configuration::Configuration::new(&deployment.initial) {
            Ok(config) => {
                for (n, database) in config.source().databases.iter().enumerate() {
                    lesson_presentation::monospace(ui, &database.name, true);
                    let values = if database.access.is_empty() {
                        "No local values; inherited/default rules apply.".into()
                    } else {
                        database
                            .access
                            .iter()
                            .map(|v| format!("olcAccess: {v}"))
                            .collect::<Vec<_>>()
                            .join("\n")
                    };
                    lesson_presentation::code_value(
                        ui,
                        ("supplied-config-values", n),
                        &values,
                        ui.available_width(),
                    );
                }
            }
            Err(e) => {
                lesson_presentation::prose(ui, &e.to_string());
            }
        }
    });
    ui.separator();
}
fn configuration_result(ui: &mut egui::Ui, result: &work::ConfigurationRun) {
    lesson_presentation::rich(
        ui,
        RichText::new(if result.preserved {
            "Configuration applied"
        } else {
            "Configuration preservation needs attention"
        })
        .strong()
        .color(if result.preserved {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    lesson_presentation::prose(ui, &result.feedback);
    for request in &result.requests {
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} · olcAccess: {} → {} values",
                request.name.text(),
                request.before.len(),
                request.after.len()
            ),
            true,
        );
    }
    controls::collapsing_header(ui, "Actual configuration values").show(ui, |ui| {
        for (n, request) in result.requests.iter().enumerate() {
            lesson_presentation::monospace(ui, &request.name.text(), true);
            let values = if request.after.is_empty() {
                "No local access values; inherited/default rules apply.".into()
            } else {
                request
                    .after
                    .iter()
                    .map(|v| format!("olcAccess: {v}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            lesson_presentation::code_value(
                ui,
                ("applied-configuration", n),
                &values,
                ui.available_width(),
            );
        }
    });
}
fn value(source: &dlms_learning_lab::ldap_query::Source, target: &str, name: &str) -> String {
    let Ok(target) = Dn::parse(target) else {
        return "Unavailable".into();
    };
    source
        .entries
        .iter()
        .find(|e| Dn::parse(&e.dn).is_ok_and(|dn| dn.equivalent(&target)))
        .and_then(|e| {
            e.attributes
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(name))
        })
        .map(|a| a.values.join(", "))
        .unwrap_or_else(|| "Not present".into())
}
fn cell(ui: &mut egui::Ui, text: &str, width: f32, strong: bool) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, 0.),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            ui.set_max_width(width);
            let text = RichText::new(text);
            lesson_presentation::rich(ui, if strong { text.strong() } else { text });
        },
    );
}
fn state_table(ui: &mut egui::Ui, r: &work::Run, p: &work::Probe) {
    let width = ui.available_width();
    let small = width < ui.text_style_height(&egui::TextStyle::Body) * 24.;
    if small {
        for f in &p.fields {
            ui.strong(&f.attribute);
            lesson_presentation::prose(
                ui,
                &format!("Before: {}", value(&r.before, &p.target, &f.attribute)),
            );
            lesson_presentation::prose(
                ui,
                &format!("After: {}", value(&r.after, &p.target, &f.attribute)),
            );
        }
    } else {
        let gap = ui.spacing().item_spacing.x;
        let field = controls::button_width(ui, "description");
        let column = ((width - field - 2. * gap) / 2.).max(1.);
        egui::Grid::new("acl-actual-fields")
            .num_columns(3)
            .spacing(ui.spacing().item_spacing)
            .show(ui, |ui| {
                for (text, w) in [("Field", field), ("Before", column), ("After", column)] {
                    cell(ui, text, w, true);
                }
                ui.end_row();
                for f in &p.fields {
                    cell(ui, &f.attribute, field, false);
                    cell(
                        ui,
                        &value(&r.before, &p.target, &f.attribute),
                        column,
                        false,
                    );
                    cell(ui, &value(&r.after, &p.target, &f.attribute), column, false);
                    ui.end_row();
                }
            });
    }
}
fn trial(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    controls::form_row(ui, "Sample directory", |ui, w| {
        let response = controls::dropdown(ui, "acl-sample", w)
            .selected_text(&t.cases[usize::from(s.selected_case)].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (index, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_label(index == usize::from(s.selected_case), &c.label);
                    record(m, format!("case-{index}"), &r, ui);
                    if r.clicked() {
                        let probe = usize::from(s.selected_probe).min(c.probes.len() - 1);
                        *notice = s.select(t, index, probe).err();
                    }
                }
            });
        record(m, "case", &response.response, ui);
    });
    let case = &t.cases[usize::from(s.selected_case)];
    controls::form_row(ui, "Change request", |ui, w| {
        let response = controls::dropdown(ui, "acl-probe", w)
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
    let p = &case.probes[usize::from(s.selected_probe)];
    lesson_presentation::monospace(
        ui,
        &format!(
            "Established caller: {}\nTarget entry: {}",
            p.actor, p.target
        ),
        true,
    );
    for f in &p.fields {
        lesson_presentation::monospace(
            ui,
            &format!("Replace {} with {}", f.attribute, f.value),
            true,
        );
    }
    let action = ui.add(button(if t.configuration.is_some() {
        "Apply and try request"
    } else {
        "Run request"
    }));
    record(m, "run", &action, ui);
    if action.clicked() {
        *notice = s.run(t).err();
    }
    let inner=ui.scope(|ui|{
  ui.strong("Actual request and directory state");
  if let Some(e)=notice{lesson_presentation::rich(ui,RichText::new(e.as_str()).color(palette(ui).error));}
  let Some(r)=&s.output else{lesson_presentation::prose(ui,if t.configuration.is_some() {"No current result. Apply and try request applies this change to the sample settings, then tries the selected contact request. Each trial starts with its original configuration and directory."}else{"No current result. Run request tries this change; Check work tests every request in every sample. Each trial starts from its original directory."});return;};
  if let Some(configuration)=&r.configuration { configuration_result(ui,configuration); ui.separator(); }
  ui.horizontal_wrapped(|ui|{
   ui.label(format!("Task requires: {}",if r.expected_allow{"Allowed"}else{"Denied"}));
   lesson_presentation::rich(ui,RichText::new(format!("Actual: {}",if r.allowed{"Allowed"}else{"Denied"})).strong().color(if r.passed{palette(ui).success}else{palette(ui).error}));
  });
  lesson_presentation::prose(ui,&r.feedback);state_table(ui,r,p);lesson_presentation::prose(ui,&r.explanation);
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
    } else if t.configuration.is_some() {
        "Check configuration"
    } else {
        "Check work"
    }));
    record(m, "check", &r, ui);
    if r.clicked() {
        *notice = s.check(t).err();
    }
    if !s.checked || s.checked_draft.as_deref() != Some(&s.draft) {
        return;
    }
    let reports = match t.reports(&s.draft) {
        Ok(v) => v,
        Err(e) => {
            lesson_presentation::prose(ui, &e);
            return;
        }
    };
    let passed = reports.iter().filter(|r| r.passed).count();
    let response = lesson_presentation::rich(
        ui,
        RichText::new(if t.configuration.is_some() {
            if passed == reports.len() {
                "Configuration and write checks verified"
            } else {
                "Configuration or write checks need attention"
            }
        } else if passed == reports.len() {
            "Write policy verified"
        } else {
            "Write policy needs attention"
        })
        .strong()
        .color(if passed == reports.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, reports.len(), response.rect));
    lesson_presentation::prose(
        ui,
        &if t.configuration.is_some() {
            format!(
                "{passed}/{} samples match. The deployed configuration and every allowed or denied contact change must meet the task.",
                reports.len()
            )
        } else {
            format!(
                "{passed}/{} sample directories match. Every required write must work; protected writes and partial multi-field changes must be blocked.",
                reports.len()
            )
        },
    );
    if t.configuration.is_some()
        && s.output.is_none()
        && let Some(configuration) = reports
            .iter()
            .flat_map(|r| &r.outcomes)
            .find_map(|r| r.configuration.as_ref())
    {
        configuration_result(ui, configuration);
    }
    if t.configuration.is_some()
        && reports
            .iter()
            .all(|r| r.outcomes.is_empty() && r.feedback == reports[0].feedback)
    {
        lesson_presentation::prose(ui, &reports[0].feedback);
    } else {
        for row in reports.iter().filter(|r| !r.passed) {
            ui.strong(format!("Needs attention · {}", row.label));
            lesson_presentation::prose(ui, &row.feedback);
        }
    }
    let all =
        controls::collapsing_header(ui, "All sample results and change outcomes").show(ui, |ui| {
            for (index, row) in reports.into_iter().enumerate() {
                let label = ui.strong(format!(
                    "{} · {}",
                    if row.passed {
                        "Pass"
                    } else {
                        "Needs attention"
                    },
                    row.label
                ));
                record(m, format!("sample-result-{index}"), &label, ui);
                for r in row.outcomes {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · {} · Expected {}; actual {}",
                            if r.passed { "Pass" } else { "Needs attention" },
                            r.label,
                            if r.expected_allow {
                                "Allowed"
                            } else {
                                "Denied"
                            },
                            if r.allowed { "Allowed" } else { "Denied" }
                        ),
                    );
                }
            }
        });
    record(m, "all-results", &all.header_response, ui);
}
impl LabApp {
    pub(super) fn ldap_access_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::LdapAccess { definition }, engine::State::LdapAccess { simulation }) =
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
                    record(&mut self.metrics.ldap_access, key, &r, ui);
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
                "Builds on full directory names and established identities. Learn ordered field/caller rules before the independent delegation task.",
                std::iter::empty(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            // A complete configuration is a document, not a narrow table cell.
            if let Some(entry) = lesson_presentation::entry(&source) {
                for (n, row) in entry.worked.iter().enumerate() {
                    lesson_presentation::rich(ui, RichText::new(&row.label).strong());
                    lesson_presentation::code_value(
                        ui,
                        ("acl-worked", &source.id, n),
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
                    record(&mut self.metrics.ldap_access, key, &r, ui);
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
            if let Some(deployment) = &task.configuration {
                let area = ui.scope(|ui| configuration_inventory(ui, deployment));
                self.metrics.ldap_access.configuration_inventory = Some(area.response.rect);
            }
            ui.push_id(("acl-workspace", &source.id, practice, challenge), |ui| {
                ui.add_enabled_ui(active, |ui| {
                    self.metrics.ldap_access.side_by_side =
                        ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 54.;
                    if self.metrics.ldap_access.side_by_side {
                        ui.columns(2, |cols| {
                            editor(
                                &mut cols[0],
                                scope,
                                task,
                                (&source, practice),
                                &mut self.metrics.ldap_access,
                                &mut self.notice,
                                &mut self.focus_input,
                            );
                            trial(
                                &mut cols[1],
                                scope,
                                task,
                                &mut self.metrics.ldap_access,
                                &mut self.notice,
                            );
                        });
                    } else {
                        editor(
                            ui,
                            scope,
                            task,
                            (&source, practice),
                            &mut self.metrics.ldap_access,
                            &mut self.notice,
                            &mut self.focus_input,
                        );
                        trial(
                            ui,
                            scope,
                            task,
                            &mut self.metrics.ldap_access,
                            &mut self.notice,
                        );
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    results(
                        ui,
                        scope,
                        task,
                        &mut self.metrics.ldap_access,
                        &mut self.notice,
                        practice,
                    );
                    if practice {
                        let c = controls::collapsing_header(ui, "Worked practice · optional").show(
                            ui,
                            |ui| {
                                lesson_presentation::monospace(ui, &task.reference, true);
                                let r = ui.add(button("Fill worked example"));
                                record(&mut self.metrics.ldap_access, "fill-example", &r, ui);
                                if r.clicked() {
                                    self.notice = scope.edit(task.reference.clone()).err();
                                }
                                lesson_presentation::prose(
                                    ui,
                                    "Fills only this practice draft without running it.",
                                );
                            },
                        );
                        record(
                            &mut self.metrics.ldap_access,
                            "worked",
                            &c.header_response,
                            ui,
                        );
                        let r = ui.add(button("Reset example"));
                        record(&mut self.metrics.ldap_access, "reset-example", &r, ui);
                        if r.clicked() {
                            *scope = work::Scope::new(task);
                            self.notice = None;
                        }
                    }
                    let c =
                        controls::collapsing_header(ui, "Help, supported subset and references")
                            .show(ui, |ui| {
                                if !practice {
                                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                                    let r = ui.add(button("Show a hint"));
                                    record(&mut self.metrics.ldap_access, "hint", &r, ui);
                                    if r.clicked() {
                                        a.hints += 1;
                                        self.dirty = true;
                                    }
                                    for h in source.hints.iter().take(a.hints) {
                                        lesson_presentation::prose(ui, h);
                                    }
                                    controls::collapsing_header(
                                        ui,
                                        "Worked approach · reveals a solution",
                                    )
                                    .show(ui, |ui| {
                                        lesson_presentation::prose(ui, &source.explanation);
                                    });
                                }
                                lesson_presentation::prose(ui, if task.configuration.is_some() {
                                        "Supported: explicit version1 LDIF Modify with one complete replace: olcAccess per request on supplied MDB entries. Values use the disclosed to/by policy subset; bounded folding/base64 and complete ordered values are supported. Global/config/frontend, schema/module/password/path changes, general server administration, fresh authentication and unlisted features are unsupported. Nothing reaches host files, services or networks."
                                    } else { work::HELP });
                                lesson_presentation::prose(ui, &source.simplifications);
                                for url in &source.sources {
                                    ui.hyperlink_to(url, url);
                                }
                            });
                    record(
                        &mut self.metrics.ldap_access,
                        "help",
                        &c.header_response,
                        ui,
                    );
                });
            });
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LdapAccess {
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
