//! Native record editing and explicit in-memory publication/lookup, no terminal shell.
use super::*;
use dlms_learning_lab::{
    dns::{self, QueryType, Server},
    dns_teaching,
    dns_workspace::{Draft, Requirement, Snapshot},
};
impl LabApp {
    pub(super) fn dns_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.dns_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_dns_view(&mut self, i: usize, view: teaching::View) {
        if let Some(l) = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap()
            .dns_learning
            .as_mut()
        {
            l.view = view;
        }
        self.help_open = false;
        self.focus_input = true;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn dns_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let scenario = self.scenarios[i].clone();
        let view = self.dns_view(i);
        if !self.progress.attempts[&scenario.id].challenge && scenario.dns_teaching.is_some() {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        self.set_dns_view(i, v);
                    }
                }
            });
        }
        ui.heading(&scenario.title);
        match view {
            teaching::View::Introduction => {
                let lesson = scenario.dns_teaching.as_ref().unwrap();
                ui.label(RichText::new(&lesson.prerequisite_note).color(palette(ui).muted));
                for concept in &lesson.concepts {
                    ui.label(RichText::new(&concept.title).strong());
                    ui.label(&concept.text);
                }
                ui.label(RichText::new("A different example to try").strong());
                ui.label(&lesson.example_description);
                ui.label(format!("Try a change: {}", lesson.try_change));
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(primary(ui, "Try this example"));
                    self.metrics.example_action = Some(r.rect);
                    if r.clicked() {
                        self.set_dns_view(i, teaching::View::Example);
                    }
                    let r = ui.add(button("Go straight to assignment"));
                    self.metrics.assignment_action = Some(r.rect);
                    if r.clicked() {
                        self.set_dns_view(i, teaching::View::Assignment);
                    }
                });
            }
            teaching::View::Example => {
                let lesson = scenario.dns_teaching.as_ref().unwrap();
                ui.label("Practice example · Ungraded; your assignment stays saved separately.");
                ui.label(&lesson.example_description);
                let mut learning = self.progress.attempts[&scenario.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dns_learning
                    .clone()
                    .unwrap();
                let before = serde_json::to_vec(&learning).unwrap();
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add(button("Fill worked example"));
                    self.metrics.insert = Some(r.rect);
                    if r.clicked() {
                        learning.fill_example(lesson).expect("validated example");
                    }
                    let r = ui.add(button("Reset example only"));
                    self.metrics.example_reset = Some(r.rect);
                    if r.clicked() {
                        learning = dns_teaching::Learning::new(lesson);
                        learning.view = teaching::View::Example;
                    }
                });
                let secondary = needs_secondary(&lesson.outcomes);
                ui.push_id(("dns-example", &scenario.id), |ui| {
                    let changed = configuration(
                        ui,
                        &mut learning.server,
                        &mut learning.draft,
                        secondary,
                        &mut self.metrics,
                        &mut self.focus_input,
                    );
                    if changed {
                        learning.checked = false;
                    }
                    let r = ui.add(primary(ui, "Check example"));
                    self.metrics.check = Some(r.rect);
                    if r.clicked() {
                        learning.checked = true;
                    }
                    if learning.checked {
                        let checks = learning.checks(lesson);
                        if checks.iter().all(|c| c.0) {
                            let r = ui.label(
                                RichText::new("Example checks pass · No assignment credit")
                                    .color(palette(ui).success),
                            );
                            self.metrics.dns_result = Some(r.rect);
                        } else {
                            for (_, text) in checks.iter().filter(|c| !c.0).take(2) {
                                let r = ui.label(
                                    RichText::new(format!("Needs attention: {text}"))
                                        .color(palette(ui).error),
                                );
                                self.metrics.dns_result.get_or_insert(r.rect);
                            }
                        }
                    }
                });
                ui.label(format!("Try a change: {}", lesson.try_change));
                let r = ui.add(button("Open full assignment"));
                self.metrics.assignment_action = Some(r.rect);
                if r.clicked() {
                    learning.view = teaching::View::Assignment;
                    self.set_dns_view(i, teaching::View::Assignment);
                }
                if serde_json::to_vec(&learning).unwrap() != before {
                    self.dirty = true;
                }
                self.progress
                    .attempts
                    .get_mut(&scenario.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .dns_learning = Some(learning);
            }
            teaching::View::Assignment => {
                ui.label(&scenario.objective);
                let engine::State::AuthoritativeDns { mut server } =
                    self.progress.attempts[&scenario.id].state.clone()
                else {
                    unreachable!()
                };
                let engine::Goal::AuthoritativeDns { requirements } = &scenario.goal else {
                    unreachable!()
                };
                let before = server.clone();
                let mut draft = self.progress.attempts[&scenario.id]
                    .workspace
                    .as_ref()
                    .unwrap()
                    .dns_draft
                    .clone()
                    .unwrap();
                let draft_before = draft.clone();
                let changed = ui
                    .push_id(("dns-assignment", &scenario.id), |ui| {
                        configuration(
                            ui,
                            &mut server,
                            &mut draft,
                            needs_secondary(requirements),
                            &mut self.metrics,
                            &mut self.focus_input,
                        )
                    })
                    .inner;
                if changed {
                    self.checked = false;
                }
                self.progress.attempts.get_mut(&scenario.id).unwrap().state =
                    engine::State::AuthoritativeDns { server };
                let w = self
                    .progress
                    .attempts
                    .get_mut(&scenario.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap();
                w.dns_draft = Some(draft.clone());
                if changed {
                    w.checked = false;
                }
                if before
                    != match &self.progress.attempts[&scenario.id].state {
                        engine::State::AuthoritativeDns { server } => server.clone(),
                        _ => unreachable!(),
                    }
                    || draft_before != draft
                {
                    self.dirty = true;
                }
                let r = ui.add_sized(
                    [240_f32.min(ui.available_width()), controls::height(ui)],
                    primary(ui, "Check configuration"),
                );
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    self.checked = true;
                    self.progress
                        .attempts
                        .get_mut(&scenario.id)
                        .unwrap()
                        .workspace
                        .as_mut()
                        .unwrap()
                        .checked = true;
                    self.dirty = true;
                }
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if self.checked {
                        let a = &self.progress.attempts[&scenario.id];
                        let pass = progress::current_solved(a, &scenario);
                        self.metrics.completion = pass;
                        if pass {
                            let r = ui.label(
                                RichText::new("Objective complete · All current checks pass.")
                                    .strong()
                                    .color(palette(ui).success),
                            );
                            self.metrics.dns_result = Some(r.rect);
                            if !a.challenge {
                                ui.label(&scenario.explanation);
                            }
                            self.next_unit_lab(ui, i);
                        } else if let engine::State::AuthoritativeDns { server } = &a.state {
                            if !draft.matches(server) {
                                let r = ui.label(RichText::new("Unsaved changes · Save the zone, then verify the published answers.").color(palette(ui).error));
                                self.metrics.dns_result = Some(r.rect);
                            } else {
                                for (_, text) in requirements
                                    .iter()
                                    .map(|r| r.check(server))
                                    .filter(|c| !c.0)
                                    .take(2)
                                {
                                    let r = ui.label(
                                        RichText::new(format!("Needs attention: {text}"))
                                            .color(palette(ui).error),
                                    );
                                    self.metrics.dns_result.get_or_insert(r.rect);
                                }
                            }
                        }
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
                self.metrics.dns_result.get_or_insert(result.response.rect);
            }
        }
    }
}
fn needs_secondary(requirements: &[Requirement]) -> bool {
    requirements.iter().any(|r| {
        matches!(
            r,
            Requirement::Published { secondary: true }
                | Requirement::AliasTo {
                    copy: Snapshot::Secondary,
                    ..
                }
                | Requirement::Records {
                    copy: Snapshot::Secondary,
                    ..
                }
                | Requirement::Resolves {
                    copy: Snapshot::Secondary,
                    ..
                }
                | Requirement::NoAnswer {
                    copy: Snapshot::Secondary,
                    ..
                }
                | Requirement::TargetAddress {
                    copy: Snapshot::Secondary,
                    ..
                }
        )
    })
}
/// Returns true only for editing/publication, not for lookup navigation.
fn configuration(
    ui: &mut egui::Ui,
    server: &mut Server,
    draft: &mut Draft,
    secondary: bool,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> bool {
    let previous = (draft.serial.clone(), draft.text.clone(), server.clone());
    ui.horizontal_wrapped(|ui| {
        ui.label(format!("Zone: {}", server.configured.origin));
        ui.label(format!("Primary serial: {}", server.primary.serial));
        if secondary {
            ui.label(format!("Secondary serial: {}", server.secondary.serial));
        }
    });
    ui.horizontal_wrapped(|ui| {
        let label = ui.label("SOA serial");
        let r = ui
            .add(
                controls::singleline(ui, &mut draft.serial)
                    .desired_width(160.)
                    .char_limit(10),
            )
            .labelled_by(label.id);
        metrics.dns_fields.push(("serial".into(), r.rect, r.id));
        metrics.inputs.push(r.rect);
    });
    ui.label(RichText::new("Resource records · OWNER. TTL IN TYPE DATA").strong());
    let label=ui.label(RichText::new("One complete record per line. Full domain names end in a dot; TTL is seconds. Save, then reload to publish.").small().color(palette(ui).muted));
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 8. + 16.;
    let text_width = (ui.available_width()
        - 2. * controls::PAD_X as f32
        - ui.spacing().scroll.allocated_width())
    .max(0.);
    let editor = egui::ScrollArea::both()
        .id_salt("dns-record-scroll")
        .min_scrolled_height(h)
        .max_height(h)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut draft.text)
                    .code_editor()
                    .lock_focus(false)
                    .font(egui::TextStyle::Monospace)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_width(text_width)
                    .desired_rows(8)
                    .char_limit(dlms_learning_lab::dns_workspace::MAX_TEXT),
            )
            .labelled_by(label.id)
        });
    metrics.editor_viewport = Some(editor.inner_rect);
    metrics.dns_editor_overflow = editor.content_size.x > editor.inner_rect.width() + 1.;
    let exceeded = draft.text.len() > dlms_learning_lab::dns_workspace::MAX_TEXT;
    workspace::bound(&mut draft.text, dlms_learning_lab::dns_workspace::MAX_TEXT);
    metrics.terminal = Some(editor.inner.rect);
    metrics
        .dns_fields
        .push(("records".into(), editor.inner.rect, editor.inner.id));
    if *focus {
        editor.inner.request_focus();
        *focus = false;
    }
    metrics.focus = ui.memory(|m| m.focused());
    if previous.0 != draft.serial || previous.1 != draft.text {
        draft.notice = None;
        draft.output.clear();
    }
    if exceeded {
        draft.notice =
            Some("Editor limit reached: keep the record text within 48,000 bytes.".into());
    }
    ui.horizontal_wrapped(|ui| {
        let r=ui.add(button("Save zone"));metrics.editor_save=Some(r.rect);if r.clicked(){draft.notice=Some(match draft.save(server){Ok(())=>"Zone saved · Reload the primary to publish these records.".into(),Err(e)=>format!("Cannot save: {e}")});}
        let r=ui.add(button("Reload primary"));metrics.dns_reload=Some(r.rect);if r.clicked(){draft.notice=Some(match server.reload_primary(){Ok(())=>"Primary published · Lookups now use the saved zone.".into(),Err(e)=>format!("Primary kept its previous copy: {}",e.message)});}
        if secondary {let r=ui.add(button("Update secondary"));metrics.dns_update=Some(r.rect);if r.clicked(){draft.notice=Some(match server.update_secondary(){Ok(dns::Transfer::Updated)=>"Secondary updated from the primary.".into(),Ok(dns::Transfer::UnchangedEqual)=>"Secondary unchanged · The primary serial is equal; increment the saved serial and reload the primary before retrying.".into(),Ok(dns::Transfer::UnchangedOlder)=>"Secondary unchanged · The primary serial is older according to DNS serial arithmetic.".into(),Err(e)=>e.message});}}
    });
    if let Some(text) = &draft.notice {
        let r = ui.add(egui::Label::new(text).wrap());
        metrics.dns_notice = Some(r.rect);
    }
    ui.add_space(4.);
    ui.label(RichText::new("Inspect a served answer").strong());
    let query_before = (draft.query_name.clone(), draft.query_type, draft.query_copy);
    let mut queried = false;
    let wide = ui.available_width() >= 700.;
    let field_width = if wide {
        300.
    } else {
        (ui.available_width() - 82.).max(100.)
    };
    let name_id = ui.make_persistent_id("dns-lookup-name");
    egui::Grid::new("dns-lookup-controls")
        .num_columns(if wide { 4 } else { 2 })
        .spacing([controls::GAP, 8.])
        .show(ui, |ui| {
            if wide {
                ui.label("Server");
                ui.label("Full domain name");
                ui.label("Record type");
                ui.label("");
                ui.end_row();
            } else {
                controls::label(ui, "Server", 70.);
            }
            if secondary {
                let r = egui::ComboBox::from_id_salt("dns-copy")
                    .selected_text(draft.query_copy.label())
                    .width(125.)
                    .show_ui(ui, |ui| {
                        let r = ui.selectable_value(
                            &mut draft.query_copy,
                            Snapshot::Primary,
                            "Primary",
                        );
                        metrics.dns_choices.push(("copy:Primary".into(), r.rect));
                        let r = ui.selectable_value(
                            &mut draft.query_copy,
                            Snapshot::Secondary,
                            "Secondary",
                        );
                        metrics.dns_choices.push(("copy:Secondary".into(), r.rect));
                        metrics.dns_popup = Some(controls::popup_content_clip(ui));
                    })
                    .response;
                r.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::WidgetType::ComboBox,
                        ui.is_enabled(),
                        "DNS lookup server",
                    )
                });
                metrics.dns_pickers.push(r.rect);
                metrics.dns_copy = Some(r.rect);
            } else {
                controls::label(ui, "Primary", 125.);
            }
            if !wide {
                ui.end_row();
                controls::label(ui, "Name", 70.);
            }
            let r = controls::cell(ui, field_width, |ui| {
                ui.add(
                    controls::singleline(ui, &mut draft.query_name)
                        .id(name_id)
                        .hint_text("host.example.test.")
                        .desired_width((field_width - 2. * controls::PAD_X as f32).max(0.))
                        .char_limit(254),
                )
            })
            .inner;
            r.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::TextEdit,
                    ui.is_enabled(),
                    "DNS lookup name",
                )
            });
            metrics.inputs.push(r.rect);
            metrics.dns_fields.push(("query".into(), r.rect, r.id));
            if !wide {
                ui.end_row();
                controls::label(ui, "Type", 70.);
            }
            let r = egui::ComboBox::from_id_salt("dns-query-type")
                .selected_text(dlms_learning_lab::dns_workspace::type_text(
                    draft.query_type,
                ))
                .width(100.)
                .show_ui(ui, |ui| {
                    for kind in [
                        QueryType::A,
                        QueryType::Aaaa,
                        QueryType::Cname,
                        QueryType::Mx,
                        QueryType::Ns,
                        QueryType::Ptr,
                    ] {
                        let label = dlms_learning_lab::dns_workspace::type_text(kind);
                        let r = ui.selectable_value(&mut draft.query_type, kind, label);
                        metrics.dns_choices.push((format!("type:{label}"), r.rect));
                    }
                    metrics.dns_popup = Some(controls::popup_content_clip(ui));
                })
                .response;
            r.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::WidgetType::ComboBox,
                    ui.is_enabled(),
                    "DNS record type",
                )
            });
            metrics.dns_pickers.push(r.rect);
            metrics.dns_type = Some(r.rect);
            if !wide {
                ui.end_row();
                ui.label("");
            }
            ui.push_id("dns-lookup-submit", |ui| {
                let r = ui.add(button("Look up"));
                metrics.probe = Some(r.rect);
                if r.clicked() {
                    queried = true;
                    draft.notice = None;
                    if let Err(e) = draft.lookup(server) {
                        draft.output = format!("Lookup error: {e}");
                    }
                }
            });
            ui.end_row();
        });
    if !queried
        && (query_before != (draft.query_name.clone(), draft.query_type, draft.query_copy)
            || previous.2 != *server)
    {
        draft.output.clear();
    }
    if !draft.output.is_empty() {
        let r = egui::ScrollArea::both()
            .id_salt("dns-lookup-output")
            .min_scrolled_height(ui.text_style_height(&egui::TextStyle::Monospace) * 3. + 16.)
            .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 6. + 16.)
            .show(ui, |ui| {
                ui.add(egui::Label::new(RichText::new(&draft.output).monospace()).selectable(true))
            });
        metrics.output = Some(r.inner_rect);
    }
    previous != (draft.serial.clone(), draft.text.clone(), server.clone())
}

#[cfg(test)]
mod no_answer_tests {
    use super::*;

    #[test]
    fn negative_secondary_outcome_exposes_selected_copy_controls() {
        let negative = |copy| Requirement::NoAnswer {
            copy,
            name: "retired.example.test.".into(),
            record_type: QueryType::Ptr,
        };
        assert!(!needs_secondary(&[negative(Snapshot::Primary)]));
        assert!(needs_secondary(&[negative(Snapshot::Secondary)]));
        assert!(needs_secondary(&[
            Requirement::Published { secondary: false },
            negative(Snapshot::Secondary),
        ]));
    }
}
