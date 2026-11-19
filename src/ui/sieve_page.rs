//! Guided and independent native Sieve editing with bounded per-frame runs.
use super::*;
use dlms_learning_lab::{sieve_source as binding, sieve_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub focus: Vec<(String, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
        self.focus.push((key.into(), r.id));
    }
}
fn editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let mut focus = false;
    legacy_recipe(ui, t, m);
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
        let r = ui.add(button("Fill worked example"));
        m.record(ui, "fill-example", &r);
        if r.clicked() {
            if let Err(e) = s.edit(t.reference.clone()) {
                *notice = Some(e);
            }
            focus = true;
        }
    }
    let label = ui.strong("Filter · Sieve editor");
    let mut draft = s.draft.clone();
    let line = ui.text_style_height(&egui::TextStyle::Monospace);
    let height = line * 10. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scroll) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("sieve-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut draft)
                        .id_salt("sieve-script")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(10)
                        .char_limit(16 * 1024),
                )
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let r = scroll.inner.labelled_by(label.id);
    let visible = r.rect.intersect(scroll.inner_rect);
    m.controls.push(("editor".into(), visible, ui.clip_rect()));
    m.fields.push(("editor".into(), visible, r.id));
    m.focus.push(("editor".into(), r.id));
    if focus {
        r.request_focus();
    }
    if r.changed()
        && let Err(e) = s.edit(draft)
    {
        *notice = Some(e);
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(!s.running, primary(ui, "Run filter"));
        m.record(ui, "run", &r);
        if r.clicked() {
            s.start_for(t);
        }
        let r = ui.add_enabled(s.running, button("Stop"));
        m.record(ui, "stop", &r);
        if r.clicked() {
            s.stop();
        }
        if practice {
            let r = ui.add(button("Reset example only"));
            m.record(ui, "reset-example", &r);
            if r.clicked() {
                *s = work::Scope::new(t);
            }
        }
    });
    if s.running {
        lesson_presentation::prose(
            ui,
            &format!(
                "Running · {}/{} samples evaluated.",
                s.results.len(),
                t.cases.len()
            ),
        );
    } else if s.stopped {
        lesson_presentation::prose(
            ui,
            "Stopped. Your draft and partial observations are retained. Run filter starts a new run.",
        );
    } else if !s.checked && s.error.is_none() {
        lesson_presentation::prose(ui, "Unchecked draft · Run filter tests every sample.");
    }
    if let Some(e) = &s.error {
        let r = lesson_presentation::rich(ui, RichText::new(e).color(palette(ui).error));
        m.record(ui, "error", &r);
    }
}
fn legacy_recipe(ui: &mut egui::Ui, t: &work::Task, m: &mut Metrics) {
    if let Some(text) = &t.legacy_recipe {
        let block = ui.scope(|ui| {
            ui.set_width(ui.available_width().floor().max(1.));
            ui.strong("Legacy procmail recipe · read only");
            lesson_presentation::monospace(ui, text, true);
            lesson_presentation::prose(ui, "Write Sieve below; this legacy recipe is never run.");
            ui.add_space(8.);
        });
        m.record(ui, "legacy-recipe", &block.response);
    }
}
fn samples(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    practice: bool,
    m: &mut Metrics,
) {
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    controls::form_row(ui, "Sample message", |ui, width| {
        let r = controls::dropdown(ui, "sieve-sample", width)
            .selected_text(&t.cases[s.selected].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let first = m.controls.len();
                for (at, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut s.selected, at, &c.label);
                    m.record(ui, &format!("sample:{}", c.id), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for row in &mut m.controls[first..] {
                    row.2 = clip;
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "sample", &r.response);
    });
    let c = &t.cases[s.selected];
    if let Some(vacation) = &t.vacation {
        let block = ui.scope(|ui| {
            // Label rounds galley extents to whole UI points. Reserve that
            // width before wrapping so fractional desktop scaling cannot grow
            // the group beyond its parent clip. Font size stays unchanged.
            ui.set_width(ui.available_width().floor().max(1.));
            ui.strong("Shared reply timeline");
            lesson_presentation::prose(ui, &format!(
                "Arrival {} of {} · {} seconds from the start · recipient {}",
                s.selected + 1, t.cases.len(), c.arrival_seconds.unwrap_or(0), vacation.user.primary));
            lesson_presentation::prose(ui,
                "Later arrivals remember earlier replies to the same sender. Run filter restarts this timeline; the separate example has its own history.");
        });
        m.record(ui, "timeline", &block.response);
    }
    if let Some(envelope) = &c.envelope {
        let envelope_block = ui.scope(|ui| {
            ui.strong("Actual delivery envelope");
            egui::Grid::new("sieve-envelope")
                .num_columns(2)
                .spacing([16., 8.])
                .show(ui, |ui| {
                    ui.label("Sender · MAIL FROM");
                    lesson_presentation::monospace(
                        ui,
                        if envelope.from.is_empty() {
                            "(null reverse path)"
                        } else {
                            &envelope.from
                        },
                        true,
                    );
                    ui.end_row();
                    ui.label("Recipient · RCPT TO");
                    lesson_presentation::monospace(ui, &envelope.to, true);
                    ui.end_row();
                });
            lesson_presentation::prose(
                ui,
                "These delivery values are separate from the visible message headers below.",
            );
        });
        m.record(ui, "envelope", &envelope_block.response);
    }
    let size = c.message().map(|m| m.octets()).unwrap_or(0);
    lesson_presentation::prose(ui, &format!("Complete represented message: {size} bytes."));
    egui::Grid::new("sieve-headers")
        .num_columns(2)
        .spacing([16., 8.])
        .show(ui, |ui| {
            ui.strong("Header");
            ui.strong("Supplied value");
            ui.end_row();
            if c.headers.is_empty() {
                ui.label("None supplied");
                ui.label("Missing fields do not match a value.");
                ui.end_row();
            }
            for (name, value) in &c.headers {
                lesson_presentation::monospace(ui, name, false);
                lesson_presentation::monospace(
                    ui,
                    if value.is_empty() { "(empty)" } else { value },
                    true,
                );
                ui.end_row();
            }
        });
    controls::collapsing_header(ui, "Show sample body").show(ui, |ui| {
        lesson_presentation::monospace(ui, &c.body, true);
    });
    lesson_presentation::prose(
        ui,
        &format!(
            "Available virtual mailboxes: {}.",
            t.mailboxes.iter().cloned().collect::<Vec<_>>().join(", ")
        ),
    );
    if t.delivery_actions && !t.redirect_targets.is_empty() {
        lesson_presentation::prose(
            ui,
            &format!(
                "Allowed virtual forwards: {}.",
                t.redirect_targets
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    ui.separator();
    let r = ui.strong("Actual run results");
    m.record(ui, "results", &r);
    if s.results.is_empty() {
        lesson_presentation::prose(
            ui,
            if s.error.is_some() {
                "This run did not produce case results. Correct the reported error, then retry."
            } else if t.vacation.is_some() {
                "Not run yet. Compare the arrival times and reply requirements, then run the filter."
            } else {
                "Not run yet. Use the requirements to predict the destination, then run the filter."
            },
        );
        return;
    }
    let passed = s.results.iter().filter(|r| r.passed).count();
    let status = if s.running {
        format!(
            "Run in progress · {}/{} cases evaluated",
            s.results.len(),
            t.cases.len()
        )
    } else if s.stopped {
        format!(
            "Stopped · {}/{} cases evaluated",
            s.results.len(),
            t.cases.len()
        )
    } else if passed == t.cases.len() {
        if practice {
            "Example checks pass".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!("Needs attention · {passed}/{} cases pass", t.cases.len())
    };
    let overall = lesson_presentation::rich(ui, RichText::new(status).strong());
    m.record(ui, "overall", &overall);
    m.result = Some((passed, t.cases.len(), overall.rect));
    for (c, r) in t.cases.iter().zip(&s.results) {
        let block=ui.scope(|ui| {
        ui.strong(format!(
            "{} · {}",
            if r.passed { "Pass" } else { "Needs attention" },
            c.label
        ));
        let describe = |set: &std::collections::BTreeSet<String>| {
            if set.is_empty() {
                "Discard, no mailbox copy".into()
            } else {
                set.iter().cloned().collect::<Vec<_>>().join(", ")
            }
        };
        let details = format!(
            "Expected: {}. Actual: {}.{}",
            if t.delivery_actions {c.expected_description()}else{describe(&c.expected)},
            if t.delivery_actions {r.description()}else{describe(&r.actual)},
            if r.implicit_keep {
                " Default INBOX keep."
            } else {
                ""
            }
        );
        if r.passed {
            let shown = controls::collapsing_header(ui, "Outcome details")
                .id_salt((&c.id, "pass"))
                .show(ui, |ui| {
                    lesson_presentation::prose(ui, &details);
                    reply_details(ui, r, m);
                });
            m.record(ui, &format!("outcome:{}", c.id), &shown.header_response);
        } else {
            lesson_presentation::prose(ui, &details);
            reply_details(ui, r, m);
            if let Some(e) = &r.error {
                lesson_presentation::prose(ui, e);
            } else {
                lesson_presentation::prose(
                    ui,
                    if let Some(hint)=t.delivery_hint(c,r) {hint} else if r.actual.len() > 1 {
                        "More than one copy remains. Check later rules and explicit keep; filing does not stop processing."
                    } else if t.address_tests {
                        "Compare the actual mailbox address and the chosen part: full address, local part or domain. A display name is not an address; an exact domain test also excludes subdomains and similar endings."
                    } else if r.implicit_keep {
                        "No filing rule matched. Check the named field, full value and condition grouping."
                    } else {
                        "The filter chose a different destination. Check the strict size boundary, whole-value test and priority order."
                    },
                );
            }
        }
        });
        if !r.passed {
            m.record(ui, &format!("failed:{}", c.id), &block.response);
            m.record(ui, "failed-case", &block.response);
        }
    }
}
fn reply_details(ui: &mut egui::Ui, r: &work::CaseResult, m: &mut Metrics) {
    if let Some(dlms_learning_lab::sieve_vacation_policy::RunResult::Sent { reply, .. }) =
        &r.vacation
    {
        let shown = controls::collapsing_header(ui, "Prepared reply · actual run result")
            .id_salt((&r.id, "vacation-reply"))
            .show(ui, |ui| {
                for (label, value) in [
                    ("From", reply.from.as_str()),
                    ("To", reply.to.as_str()),
                    ("Subject", reply.subject.as_str()),
                    ("Auto-Submitted", "auto-replied"),
                ] {
                    controls::form_row(ui, label, |ui, _| {
                        lesson_presentation::monospace(ui, value, true);
                    });
                }
                ui.strong("Reply message");
                lesson_presentation::monospace(ui, &reply.body, true);
            });
        m.record(
            ui,
            &format!("reply-toggle:{}", r.id),
            &shown.header_response,
        );
        if let Some(body) = shown.body_response {
            m.record(ui, "reply-body", &body);
        }
    }
}
impl LabApp {
    pub(super) fn sieve_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (
            engine::Goal::SieveFiltering { definition },
            engine::State::SieveFiltering { simulation },
        ) = (&source.goal, &original.state)
        else {
            return;
        };
        let first = egui::Id::new(("sieve-first-frame", &source.id, original.visited));
        if !ui
            .ctx()
            .data(|d| d.get_temp::<bool>(first))
            .unwrap_or(false)
        {
            if let Err(e) = binding::context(original, &source) {
                lesson_presentation::prose(ui, &e);
                return;
            }
            self.focus_input = false;
            ui.ctx().data_mut(|d| d.insert_temp(first, true));
        }
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
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.sieve.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        if view != before.view {
            next.assignment.stop();
            next.practice.stop();
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            legacy_recipe(ui, &definition.example, &mut self.metrics.sieve);
            lesson_presentation::worked(
                ui,
                "Separate worked filter · compare the conditions with the supplied samples.",
            );
            let r = ui.scope(|ui| {
                lesson_presentation::monospace(ui, &definition.example.reference, true);
                lesson_presentation::worked_rows(ui, &source);
            });
            self.metrics.sieve.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.sieve.record(ui, "try", &r);
            self.metrics.example_action = Some(r.rect);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            if !practice {
                lesson_presentation::assignment(ui, &source);
            }
            let (t, s) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.assignment, &mut next.assignment)
            };
            ui.push_id(("sieve-workspace", &source.id, practice), |ui| {
                ui.add_enabled_ui(active, |ui| {
                    if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 45. {
                        ui.columns(2, |cols| {
                            editor(
                                &mut cols[0],
                                t,
                                s,
                                &source,
                                practice,
                                &mut self.metrics.sieve,
                                &mut self.notice,
                            );
                            samples(
                                &mut cols[1],
                                t,
                                s,
                                &source,
                                practice,
                                &mut self.metrics.sieve,
                            );
                        });
                    } else {
                        editor(
                            ui,
                            t,
                            s,
                            &source,
                            practice,
                            &mut self.metrics.sieve,
                            &mut self.notice,
                        );
                        ui.add_space(12.);
                        samples(ui, t, s, &source, practice, &mut self.metrics.sieve);
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    if active && s.running {
                        s.tick(t);
                        ui.ctx().request_repaint();
                    }
                });
            });
            controls::collapsing_header(ui, "Optional help and Sieve reference").show(ui, |ui| {
                if !practice {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    if ui
                        .add_enabled(
                            active && a.hints < source.hints.len(),
                            button("Show a hint"),
                        )
                        .clicked()
                    {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for hint in source.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, hint);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals the assignment")
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                            lesson_presentation::monospace(ui, &t.reference, true);
                        });
                }
                lesson_presentation::prose(ui, &source.simplifications);
                for url in &source.sources {
                    ui.hyperlink_to(url, url);
                }
            });
        }
        if active && next != before {
            match next.validate() {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::SieveFiltering {
                            simulation: Box::new(next),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        let ready = matches!(&self.progress.attempts[&source.id].state,engine::State::SieveFiltering{simulation}if simulation.assignment.checked);
        self.metrics.completion = view == teaching::View::Assignment
            && ready
            && progress::current_solved(&self.progress.attempts[&source.id], &source);
        if self.metrics.completion {
            self.next_unit_lab(ui, i);
        }
    }
}
