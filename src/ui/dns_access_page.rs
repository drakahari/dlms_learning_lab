//! Native literal configuration and simulated query/transfer tools. No terminal.
use super::*;
use dlms_learning_lab::{dns::QueryType, dns_access_workspace as work};
use work::{Action, Task, session::Draft};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub summary: Option<String>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
    fn field(&mut self, key: &str, r: &egui::Response, clip: egui::Rect) {
        self.fields.push((key.into(), r.rect, r.id, clip));
    }
}
fn guidance(ui: &mut egui::Ui, s: &pack::Scenario, step: usize, practice: bool) {
    if practice {
        lesson_presentation::practice_step(ui, s, step);
    }
}
fn run(draft: &mut Draft, task: &Task, action: Action, notice: &mut Option<String>) {
    // Failed operations already have replayed last-action feedback.
    *notice = draft.run(task, action).err();
}
fn field(
    ui: &mut egui::Ui,
    label: &str,
    value: &mut String,
    limit: usize,
    key: &str,
    metrics: &mut Metrics,
    notice: &mut Option<String>,
) -> bool {
    let mut text = value.clone();
    let cue = if key.ends_with("client") {
        "Dotted-decimal IPv4"
    } else if key.ends_with("zone") {
        "Full zone name"
    } else {
        "Full DNS name"
    };
    let r = controls::form_row(ui, label, |ui, width| {
        ui.add(
            controls::singleline_fitting(ui, &mut text, width)
                .char_limit(limit)
                .hint_text(cue),
        )
    })
    .inner;
    r.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::TextEdit, r.enabled(), label));
    metrics.field(key, &r, ui.clip_rect());
    let accepted = text.len() <= limit;
    if r.changed() {
        if accepted {
            *value = text;
        } else {
            *notice = Some(format!("{label}: keep this input within {limit} bytes."));
        }
    }
    accepted && ui.is_enabled() && r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
}
fn editor(
    ui: &mut egui::Ui,
    s: &pack::Scenario,
    t: &Task,
    d: &mut Draft,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    guidance(ui, s, 0, practice);
    let label = ui.strong("named.conf · simulated configuration");
    if practice {
        let r = ui.add(button("Fill worked configuration"));
        m.record(ui, "fill", &r);
        if r.clicked()
            && let Some(Action::Save { config }) = t
                .reference
                .iter()
                .find(|a| matches!(a, Action::Save { .. }))
        {
            *notice = d.set_editor(config.0.clone()).err();
        }
    }
    let mut text = d.editor.0.clone();
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 14. + 2. * f32::from(controls::PAD_Y);
    let outer_size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scrolled) = controls::fixed_viewport(ui, outer_size, |ui| {
        egui::ScrollArea::both()
            .id_salt("dns-policy-editor-scroll")
            .max_height(outer_size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt("dns-policy-editor")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(14)
                        .char_limit(8192),
                )
                .labelled_by(label.id)
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let r = scrolled.inner;
    m.field("editor", &r, scrolled.inner_rect.intersect(ui.clip_rect()));
    if r.changed() {
        *notice = d.set_editor(text).err();
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save configuration"));
        m.record(ui, "save", &r);
        if r.clicked() {
            run(
                d,
                t,
                Action::Save {
                    config: d.editor.clone(),
                },
                notice,
            );
        }
        let r = ui.add(button("Reload configuration"));
        m.record(ui, "reload", &r);
        if r.clicked() {
            run(d, t, Action::Reload {}, notice);
        }
    });
    match d.world(t) {
        Ok(world) => {
            ui.label(if d.editor.0 == world.saved_text() {
                "Editor matches the saved file"
            } else {
                "Editor has unsaved changes"
            });
            ui.label(if world.saved() == world.live() {
                "Saved access rules are applied"
            } else {
                "Saved access rules are waiting for reload"
            });
        }
        Err(error) => {
            lesson_presentation::prose(ui, &error);
        }
    }
}
fn query(
    ui: &mut egui::Ui,
    s: &pack::Scenario,
    t: &Task,
    d: &mut Draft,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    guidance(ui, s, 1, practice);
    ui.strong("Test a simulated DNS query");
    let mut enter = field(
        ui,
        "Client IPv4",
        &mut d.query_client.0,
        16,
        "query-client",
        m,
        notice,
    );
    enter |= field(
        ui,
        "Name to ask for",
        &mut d.query_name.0,
        254,
        "query-name",
        m,
        notice,
    );
    controls::form_row(ui, "Record type", |ui, width| {
        let r = controls::dropdown(ui, "dns-policy-type", width)
            .selected_text(dlms_learning_lab::dns_workspace::type_text(d.query_type))
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for kind in [
                    QueryType::A,
                    QueryType::Aaaa,
                    QueryType::Cname,
                    QueryType::Mx,
                    QueryType::Ns,
                    QueryType::Ptr,
                ] {
                    let r = ui.selectable_value(
                        &mut d.query_type,
                        kind,
                        dlms_learning_lab::dns_workspace::type_text(kind),
                    );
                    m.record(ui, &format!("type-{kind:?}"), &r);
                }
            })
            .response;
        m.record(ui, "query-type", &r);
    });
    let r = ui.checkbox(&mut d.recursive, "Ask for a recursive lookup if needed");
    m.record(ui, "recursive", &r);
    let r = ui.add(button("Run query"));
    m.record(ui, "query", &r);
    if ui.is_enabled() && (r.clicked() || enter) {
        match d.query_action() {
            Ok(action) => run(d, t, action, notice),
            Err(error) => *notice = Some(error),
        }
    }
    let transfers = t
        .required
        .transfers
        .iter()
        .any(|r| r.synchronized || !r.clients.intervals().is_ok_and(|v| v.is_empty()));
    controls::collapsing_header(ui, "Test a zone transfer")
        .default_open(transfers)
        .show(ui, |ui| {
            if let Ok(world) = d.world(t) {
                for zone in world.zones() {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{} · primary serial {} · secondary {} at {}",
                            zone.server.primary.origin,
                            zone.server.primary.serial,
                            zone.server.secondary.serial,
                            zone.secondary_client
                        ),
                    );
                }
            }

            let mut enter = field(
                ui,
                "Request from IPv4",
                &mut d.transfer_client.0,
                16,
                "transfer-client",
                m,
                notice,
            );
            enter |= field(
                ui,
                "Zone to copy",
                &mut d.transfer_zone.0,
                254,
                "transfer-zone",
                m,
                notice,
            );
            let r = ui.add(button("Request transfer"));
            m.record(ui, "transfer", &r);
            if ui.is_enabled() && (r.clicked() || enter) {
                match d.transfer_action() {
                    Ok(action) => run(d, t, action, notice),
                    Err(error) => *notice = Some(error),
                }
            }
        });
    let output = ui.scope(|ui| {
        ui.strong("Actual last action");
        match d.last_output(t) {
            Ok(Some(result)) => {
                let text = match result {
                    Ok(text) => text,
                    Err(error) => format!("Action did not run: {error}"),
                };
                controls::nested_scroll(
                    ui,
                    egui::ScrollArea::vertical()
                        .id_salt("dns-policy-last-output")
                        .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 12.)
                        .auto_shrink([false, true]),
                    |ui| lesson_presentation::monospace(ui, &text, true),
                );
            }
            Ok(None) => {
                ui.label("No action has run in this draft yet.");
            }
            Err(error) => {
                lesson_presentation::prose(ui, &error);
            }
        }
    });
    m.record(ui, "output", &output.response);
}
fn copies(ui: &mut egui::Ui, t: &Task, d: &Draft, m: &mut Metrics) {
    let Ok(world) = d.world(t) else {
        return;
    };
    controls::collapsing_header(
        ui,
        "Inspect supplied primary and secondary copies · optional",
    )
    .show(ui, |ui| {
        for (index, zone) in world.zones().iter().enumerate() {
            let r = ui.scope(|ui| {
                ui.strong(&zone.server.primary.origin);
                for (label, value) in [
                    ("Virtual file", zone.file.clone()),
                    ("Secondary IPv4", zone.secondary_client.to_string()),
                    ("Primary serial", zone.server.primary.serial.to_string()),
                    ("Secondary serial", zone.server.secondary.serial.to_string()),
                ] {
                    controls::form_row(ui, label, |ui, _| {
                        lesson_presentation::monospace(ui, &value, true)
                    });
                }
                ui.label(if zone.server.primary == zone.server.secondary {
                    "Copies match"
                } else {
                    "The secondary has a different copy"
                });
                for (label, copy) in [
                    ("Primary records", &zone.server.primary),
                    ("Secondary records", &zone.server.secondary),
                ] {
                    controls::collapsing_header(ui, label).show(ui, |ui| {
                        for record in &copy.records {
                            lesson_presentation::monospace(
                                ui,
                                &dlms_learning_lab::dns_workspace::record_text(record),
                                true,
                            );
                        }
                    });
                }
            });
            m.record(ui, &format!("copy-{index}"), &r.response);
        }
    });
}
fn feedback(ui: &mut egui::Ui, t: &Task, d: &Draft, m: &mut Metrics) {
    if !d.attempted {
        ui.label("Not checked");
        return;
    }
    let rows = match d.checks(t) {
        Ok(rows) => rows,
        Err(error) => {
            ui.label(error);
            return;
        }
    };
    let failed = rows.iter().filter(|r| !r.correct).count();
    let summary = if failed == 0 {
        "All access requirements pass".into()
    } else {
        format!("Needs attention · {failed} requirements fail")
    };
    m.summary = Some(summary.clone());
    let r = lesson_presentation::rich(
        ui,
        RichText::new(summary).color(if failed == 0 {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.record(ui, "result", &r);
    let render = |ui: &mut egui::Ui, index: usize, row: work::Check, m: &mut Metrics| {
        let r = ui.push_id(index, |ui| {
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "{} · {}: {}",
                    if row.correct {
                        "Meets requirement"
                    } else {
                        "Needs attention"
                    },
                    row.label,
                    row.feedback
                ))
                .color(if row.correct {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            if !row.correct
                && let (Some(expected), Some(actual)) = (row.expected, row.actual)
            {
                controls::collapsing_header(ui, "Expected versus actual result").show(ui, |ui| {
                    ui.strong("Expected");
                    lesson_presentation::monospace(ui, &expected, true);
                    ui.strong("Actual");
                    lesson_presentation::monospace(ui, &actual, true);
                });
            }
        });
        m.record(ui, &format!("feedback-{index}"), &r.response);
    };
    for (index, row) in rows.iter().enumerate().filter(|(_, r)| !r.correct) {
        render(ui, index, row.clone(), m);
    }
    let passed = rows.len() - failed;
    if passed > 0 {
        let header = controls::collapsing_header(ui, &format!("Show {passed} passing checks"));
        let response = header.show(ui, |ui| {
            for (index, row) in rows.into_iter().enumerate().filter(|(_, r)| r.correct) {
                render(ui, index, row, m);
            }
        });
        m.record(ui, "passing-checks", &response.header_response);
    }
}
impl LabApp {
    pub(super) fn dns_access_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let def =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(error) => {
                    self.notice = Some(error);
                    return;
                }
            };
        let engine::State::DnsAccess { simulation } = &attempt.state else {
            return;
        };
        let before = (**simulation).clone();
        let mut next = before.clone();
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
        self.metrics.dns_access = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.dns_access.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i,view as u8),|ui|{
            if view==teaching::View::Introduction{
                lesson_presentation::concepts(ui,&source,"Builds on authoritative DNS names, client IPv4 ranges and separate primary/secondary copies.",Vec::<(String,String)>::new());
                lesson_presentation::worked(ui,"Different supplied names and client ranges demonstrate the access boundaries before independent work.");for(number,r)in lesson_presentation::worked_command_rows(ui,&source).iter().enumerate(){self.metrics.dns_access.record(ui,&format!("worked-value-{number}"),r);}
                ui.horizontal_wrapped(|ui|{for(choice,key,label)in[(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));self.metrics.dns_access.record(ui,key,&r);if r.clicked(){next.view=choice;self.lesson_scroll_reset=true;}}});
            }else{
                let practice=view==teaching::View::Example;if !practice{lesson_presentation::assignment(ui,&source);}
                let(task,draft)=if practice{(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
                ui.add_enabled_ui(active,|ui|{
                    if ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*48.{ui.columns(2,|c|{editor(&mut c[0],&source,task,draft,practice,&mut self.metrics.dns_access,&mut self.notice);query(&mut c[1],&source,task,draft,practice,&mut self.metrics.dns_access,&mut self.notice);});}else{editor(ui,&source,task,draft,practice,&mut self.metrics.dns_access,&mut self.notice);ui.separator();query(ui,&source,task,draft,practice,&mut self.metrics.dns_access,&mut self.notice);}
                    copies(ui,task,draft,&mut self.metrics.dns_access);ui.separator();guidance(ui,&source,2,practice);
                    let r=ui.add(button(if practice{"Check example access"}else{"Check access"}).fill(palette(ui).accent));self.metrics.dns_access.record(ui,"check",&r);if r.clicked(){self.notice=draft.check(task).err();}
                    feedback(ui,task,draft,&mut self.metrics.dns_access);
                    if practice{let r=ui.add(button("Reset example only"));self.metrics.dns_access.record(ui,"reset-example",&r);if r.clicked(){match Draft::new(task){Ok(fresh)=>{*draft=fresh;self.notice=None;},Err(error)=>self.notice=Some(error)}}}
                    controls::collapsing_header(ui,"Supported configuration and help · optional").show(ui,|ui|{lesson_presentation::prose(ui,work::HELP);let attempt=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(attempt.hints<source.hints.len(),button("Show a hint"));if r.clicked(){attempt.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(attempt.hints){lesson_presentation::prose(ui,hint);}lesson_presentation::prose(ui,&source.simplifications);controls::collapsing_header(ui,"Worked approach · reveals one method").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));});
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::DnsAccess {
                simulation: Box::new(next),
            };
            self.completion_index.forget(&source.id);
            self.dirty = true;
        }
        self.metrics.completion = view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            lesson_presentation::prose(ui, &source.explanation);
            self.next_unit_lab(ui, i);
        }
    }
}
