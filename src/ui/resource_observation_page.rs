//! Bounded recorded-host terminal beside a learner-authored evidence report.
use super::*;
use dlms_learning_lab::resource_observation_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub output: Option<egui::Rect>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn stamp(t: &work::Task, index: usize) -> String {
    let o = &t.recording.observations[index].counters;
    format!("{} · {}.{:03} s", o.boot, o.at_ms / 1000, o.at_ms % 1000)
}
fn row_name(t: &work::Task, reading: &work::Reading, row: usize) -> String {
    let (from, to) = reading.span(row, t).unwrap();
    let end = t.recording.observations[to].counters.at_ms;
    match from {
        Some(i) => format!(
            "Row {} · {}–{} ms interval",
            row + 1,
            t.recording.observations[i].counters.at_ms,
            end
        ),
        None if matches!(reading, work::Reading::Vmstat(_)) => {
            format!("Row {} · since boot to {end} ms", row + 1)
        }
        _ => format!("Memory snapshot · {end} ms"),
    }
}
fn terminal(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Draft,
    m: &mut Metrics,
    active: bool,
    practice: bool,
    source: &pack::Scenario,
) {
    lesson_presentation::rich(
        ui,
        RichText::new("Recorded host · simulated terminal").strong(),
    );
    ui.horizontal_wrapped(|ui| {
        ui.label("Observation");
        let response = ui
            .add_enabled_ui(active, |ui| {
                controls::dropdown(
                    ui,
                    "observation-cursor",
                    ui.available_size_before_wrap().x.min(360.),
                )
                .selected_text(stamp(t, s.cursor))
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let first = m.choices.len();
                    for i in 0..t.recording.observations.len() {
                        let r = ui.selectable_value(&mut s.cursor, i, stamp(t, i));
                        m.choices
                            .push((format!("observation-{i}"), r.rect, ui.clip_rect()));
                    }
                    let clip = controls::popup_content_clip(ui);
                    for choice in &mut m.choices[first..] {
                        choice.2 = clip;
                    }
                })
                .response
            })
            .inner;
        record(m, "observation", &response, ui);
    });
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    ui.label("Command");
    let mut submit = false;
    ui.horizontal_wrapped(|ui| {
        let width =
            (ui.available_width() - controls::button_width(ui, "Run command") - controls::GAP)
                .max(120.)
                .min(ui.available_width());
        let r = ui.add_enabled(
            active,
            controls::singleline_fitting(ui, &mut s.input, width)
                .hint_text("Supported simulated command"),
        );
        m.fields
            .push(("command".into(), r.rect, r.id, ui.clip_rect()));
        submit = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let r = ui.add_enabled(active && !s.input.trim().is_empty(), button("Run command"));
        record(m, "run", &r, ui);
        submit |= r.clicked();
    });
    if submit && active && !s.input.trim().is_empty() {
        let probe = work::Probe {
            start: s.cursor,
            command: s.input.clone(),
        };
        if let Err(e) = s.apply(t, &work::Action::Run { probe }) {
            lesson_presentation::prose(ui, &e);
        }
    }
    if let Some(probe) = s.history.last() {
        lesson_presentation::prose(
            ui,
            &format!("Last command result · {}", stamp(t, probe.start)),
        );
        ui.monospace(format!("$ {}", probe.command));
        match t.probe(probe) {
            Ok(output) => {
                let height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.;
                let frame = egui::Frame::new()
                    .fill(palette(ui).background)
                    .inner_margin(12.)
                    .show(ui, |ui| {
                        egui::ScrollArea::both()
                            .id_salt("recorded-command-output")
                            .max_height(height)
                            .show(ui, |ui| {
                                ui.add(
                                    egui::Label::new(
                                        RichText::new(output.text(t).unwrap()).monospace(),
                                    )
                                    .wrap_mode(egui::TextWrapMode::Extend),
                                );
                            });
                    });
                m.output = Some(frame.response.rect);
                ui.label("Result row");
                let mut row = s.result_row;
                let response = ui
                    .add_enabled_ui(active, |ui| {
                        controls::dropdown(ui, "recorded-output-row", ui.available_width())
                            .selected_text(row_name(t, &output, row))
                            .show_ui(ui, |ui| {
                                controls::fit_popup_contents(ui);
                                let first = m.choices.len();
                                for i in 0..output.len() {
                                    let r =
                                        ui.selectable_value(&mut row, i, row_name(t, &output, i));
                                    m.choices.push((format!("row-{i}"), r.rect, ui.clip_rect()));
                                }
                                let clip = controls::popup_content_clip(ui);
                                for choice in &mut m.choices[first..] {
                                    choice.2 = clip;
                                }
                            })
                            .response
                    })
                    .inner;
                record(m, "result-row", &response, ui);
                if row != s.result_row {
                    s.select_row(t, row).unwrap();
                }
            }
            Err(error) => {
                let r = ui.add(
                    egui::Label::new(
                        RichText::new(format!("Command failed · {error}")).color(palette(ui).error),
                    )
                    .wrap(),
                );
                m.output = Some(r.rect);
            }
        }
    } else {
        lesson_presentation::prose(
            ui,
            "Run a supported command to inspect this recording. Results appear here.",
        );
    }
    controls::collapsing_header(ui, "Earlier commands").show(ui, |ui| {
        for p in s.history.iter().rev().skip(1) {
            ui.monospace(format!("{} · {}", stamp(t, p.start), p.command));
        }
    });
}
fn report(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Draft,
    m: &mut Metrics,
    active: bool,
    practice: bool,
    source: &pack::Scenario,
) {
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    lesson_presentation::rich(ui, RichText::new("Your measurement report").strong());
    let rows = if s.checked { Some(t.check(s)) } else { None };
    let unit_font = egui::TextStyle::Body.resolve(ui.style());
    let text_color = palette(ui).text;
    let units_width = ui
        .fonts_mut(|f| {
            f.layout_no_wrap("processes".into(), unit_font.clone(), text_color)
                .size()
                .x
        })
        .ceil();
    let attach_width = controls::button_width(ui, "Attach row");
    let minimum_field = ui
        .fonts_mut(|f| {
            f.layout_no_wrap("Whole number".into(), unit_font, text_color)
                .size()
                .x
        })
        .ceil()
        + 2. * f32::from(controls::PAD_X);
    let field_width = (ui.available_width() - units_width - attach_width - 2. * controls::GAP)
        .clamp(minimum_field, 160f32.max(minimum_field));
    let group_width = field_width + units_width + attach_width + 2. * controls::GAP;
    for (index, rule) in t.rules.iter().enumerate() {
        ui.push_id(&rule.id, |ui| {
            controls::form_row_group(ui, &rule.label, group_width, |ui, _| {
                ui.horizontal_wrapped(|ui| {
                    let value = &mut s
                        .claims
                        .iter_mut()
                        .find(|c| c.rule == rule.id)
                        .unwrap()
                        .value;
                    let r = ui.add_enabled(
                        active,
                        controls::singleline_fitting(
                            ui,
                            value,
                            field_width.min(ui.available_size_before_wrap().x),
                        )
                        .hint_text("Whole number"),
                    );
                    m.fields
                        .push((rule.id.clone(), r.rect, r.id, ui.clip_rect()));
                    if r.changed() {
                        s.checked = false;
                    }
                    ui.allocate_ui_with_layout(
                        egui::vec2(units_width, controls::height(ui)),
                        egui::Layout::left_to_right(egui::Align::Center),
                        |ui| {
                            ui.set_width(units_width);
                            ui.label(rule.metric.units());
                        },
                    );
                    let eligible = s.history.last().is_some_and(|p| t.probe(p).is_ok());
                    let r = ui.add_enabled(active && eligible, button("Attach row"));
                    record(m, &format!("attach-{}", rule.id), &r, ui);
                    if r.clicked() {
                        s.apply(
                            t,
                            &work::Action::Attach {
                                rule: rule.id.clone(),
                                row: s.result_row,
                            },
                        )
                        .unwrap();
                    }
                });
            });
            let claim = s.claims.iter().find(|c| c.rule == rule.id).unwrap();
            if let Some(e) = &claim.evidence {
                let reading = t.probe(&e.probe).unwrap();
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "Evidence: {} · {}",
                        stamp(t, e.probe.start),
                        row_name(t, &reading, e.row)
                    ),
                );
            } else {
                lesson_presentation::prose(ui, "No evidence attached.");
            }
            if s.checked
                && let Some(rows) = &rows
            {
                let (ok, text) = &rows[index];
                let text = text
                    .strip_prefix(&format!("{}: ", rule.label))
                    .unwrap_or(text);
                let response = lesson_presentation::rich(
                    ui,
                    RichText::new(format!(
                        "{} · {text}",
                        if *ok { "Matches" } else { "Needs attention" }
                    ))
                    .color(if *ok {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }),
                );
                record(m, &format!("feedback-{}", rule.id), &response, ui);
            }
        });
        ui.add_space(8.);
    }
    let r = ui.add_enabled(
        active,
        primary(
            ui,
            if practice {
                "Check example report"
            } else {
                "Check report"
            },
        ),
    );
    record(m, "check", &r, ui);
    if r.clicked() {
        s.check(t).unwrap();
    }
    if s.checked {
        let rows = t.check(s);
        let failed = rows.iter().filter(|r| !r.0).count();
        let text = if failed == 0 {
            format!(
                "Report verified · All {} measurements match their evidence.",
                rows.len()
            )
        } else {
            format!(
                "Report needs attention · {failed} of {} rows need another look.",
                rows.len()
            )
        };
        let r = lesson_presentation::rich(ui, RichText::new(text).strong());
        m.result = Some((rows.len(), failed, r.rect));
        controls::collapsing_header(ui, "Report preview").show(ui, |ui| {
            ui.monospace(s.report(t).unwrap());
        });
    }
}
impl LabApp {
    pub(super) fn resource_observation_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(attempt, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::Goal::ResourceObservations { definition } = &source.goal else {
            return;
        };
        let engine::State::ResourceObservations { simulation } = &attempt.state else {
            return;
        };
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
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    record(&mut self.metrics.resource_observation, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on Linux command input, processes and interpreting numeric reports.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "A different supplied recording demonstrates the method. Try it with the native terminal before the independent assignment.",
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.resource_observation, key, &r, ui);
                    if r.clicked() {
                        next.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if !practice {
                lesson_presentation::assignment(ui, &source);
            }
            let (t, s) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            ui.push_id((i, practice), |ui| {
                if ui.available_width() > 1120. {
                    let width = ui.available_width();
                    let left = (width - controls::GAP) * 0.68;
                    let right = width - controls::GAP - left;
                    ui.horizontal_top(|ui| {
                        ui.allocate_ui_with_layout(
                            egui::vec2(left, 0.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(left);
                                terminal(
                                    ui,
                                    t,
                                    s,
                                    &mut self.metrics.resource_observation,
                                    active,
                                    practice,
                                    &source,
                                );
                            },
                        );
                        ui.allocate_ui_with_layout(
                            egui::vec2(right, 0.),
                            egui::Layout::top_down(egui::Align::Min),
                            |ui| {
                                ui.set_width(right);
                                report(
                                    ui,
                                    t,
                                    s,
                                    &mut self.metrics.resource_observation,
                                    active,
                                    practice,
                                    &source,
                                );
                            },
                        );
                    });
                } else {
                    terminal(
                        ui,
                        t,
                        s,
                        &mut self.metrics.resource_observation,
                        active,
                        practice,
                        &source,
                    );
                    ui.separator();
                    report(
                        ui,
                        t,
                        s,
                        &mut self.metrics.resource_observation,
                        active,
                        practice,
                        &source,
                    );
                }
            });
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(
                    &mut self.metrics.resource_observation,
                    "reset-example",
                    &r,
                    ui,
                );
                if r.clicked() {
                    next.practice = work::Draft::new(&definition.example);
                }
            }
            controls::collapsing_header(ui, "Help and references").show(ui, |ui| {
                lesson_presentation::prose(ui, work::HELP);
                let a = self.progress.attempts.get_mut(&source.id).unwrap();
                let r = ui.add_enabled(
                    active && a.hints < source.hints.len(),
                    button("Show a hint"),
                );
                if r.clicked() {
                    a.hints += 1;
                    self.dirty = true;
                }
                for hint in source.hints.iter().take(a.hints) {
                    lesson_presentation::prose(ui, hint);
                }
                controls::collapsing_header(ui, "Worked approach · reveals one solution").show(
                    ui,
                    |ui| {
                        lesson_presentation::prose(ui, &source.explanation);
                    },
                );
                for url in &source.sources {
                    ui.hyperlink_to(url, url);
                }
            });
        }
        if next != before && active {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::ResourceObservations {
                            simulation: Box::new(next),
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
