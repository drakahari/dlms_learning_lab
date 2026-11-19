//! Opt-in native cache headers and actual deterministic request traces.
use super::*;
use dlms_learning_lab::{engine::State, http_cache as model, http_cache_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
    pub feedback: Vec<String>,
    pub cases: Vec<(String, bool)>,
    pub guidance_order: Vec<usize>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: impl Into<String>, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn delivery(d: model::Delivery) -> &'static str {
    match d {
        model::Delivery::Cached => "Reused stored response",
        model::Delivery::Validated => "Validated stored response (304)",
        model::Delivery::Downloaded => "Downloaded response (200)",
        model::Delivery::Unavailable => "No usable response",
    }
}
fn expected(e: &work::Expected) -> String {
    let how = e
        .delivery
        .iter()
        .map(|d| d.label())
        .collect::<Vec<_>>()
        .join(" or ");
    let version = match &e.body {
        work::Body::Unavailable => "no body".into(),
        work::Body::Version { etag, .. } => format!("version {etag}"),
    };
    match e.storage {
        work::Storage::Any => format!("{how}; {version}"),
        work::Storage::Required => format!("{how}; {version}; cached copy required"),
        work::Storage::Forbidden => format!("{how}; {version}; no cached copy allowed"),
    }
}
fn actual_label(a: &model::Observation) -> String {
    let body = a
        .etag
        .as_ref()
        .map_or_else(|| "no response body".into(), |tag| format!("version {tag}"));
    format!(
        "{}; {body}; {}",
        delivery(a.delivery),
        if a.stored {
            "cached copy present"
        } else {
            "no cached copy"
        }
    )
}

fn configuration(
    ui: &mut egui::Ui,
    s: &pack::Scenario,
    t: &work::Task,
    d: &mut work::Draft,
    practice: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    if practice {
        m.guidance_order.push(0);
        lesson_presentation::practice_step(ui, s, 0);
    }
    ui.strong("Origin response headers · simulated configuration");
    let mut headers = d.headers.clone();
    for (index, r) in t.resources.iter().enumerate() {
        ui.push_id(&r.path, |ui| {
            let label = ui.label(&r.label);
            controls::form_row(ui, RichText::new(&r.path).monospace(), |ui, width| {
                let response = ui
                    .add(
                        controls::singleline_fitting(ui, &mut headers[index].value, width)
                            .hint_text("Cache-Control value")
                            .char_limit(1024)
                            .id_salt("response-control"),
                    )
                    .labelled_by(label.id);
                m.fields.push((
                    format!("header-{index}"),
                    response.rect,
                    response.id,
                    ui.clip_rect(),
                ));
                m.record(ui, format!("header-{index}"), &response);
            });
        });
        ui.add_space(controls::GAP);
    }
    if headers != d.headers {
        match d.edit(t, headers) {
            Ok(()) => *notice = None,
            Err(e) => *notice = Some(e),
        }
    }
    if practice {
        let r = ui.add(button("Fill worked headers"));
        m.record(ui, "fill", &r);
        if r.clicked() {
            *notice = d.edit(t, t.reference_headers()).err();
        }
        m.guidance_order.push(1);
        lesson_presentation::practice_step(ui, s, 1);
    }
    let r = ui.add(primary(
        ui,
        if practice {
            "Run example traces"
        } else {
            "Check caching policy"
        },
    ));
    m.record(ui, "check", &r);
    if r.clicked() {
        match d.check(t) {
            Ok(report) => {
                *notice = None;
                if let Some(failed) = report
                    .traces
                    .iter()
                    .find(|tr| tr.steps.iter().any(|step| !step.passed))
                {
                    d.selected_trace = failed.id.clone();
                }
            }
            Err(e) => *notice = Some(e),
        }
    }
}
fn results(
    ui: &mut egui::Ui,
    s: &pack::Scenario,
    t: &work::Task,
    d: &mut work::Draft,
    practice: bool,
    m: &mut Metrics,
) {
    let report = if d.checked() { Some(t.report(d)) } else { None };
    let good = matches!(&report,Some(Ok(r))if r.passed());
    let summary = match &report {
        None => "Not checked since editing".into(),
        Some(Err(e)) => e.clone(),
        Some(Ok(r)) if r.passed() => "All declared cache requirements verified".into(),
        Some(Ok(r)) if !r.configuration_errors.is_empty() => {
            "Headers need attention before the traces can run".into()
        }
        Some(Ok(r)) => format!(
            "Needs attention · {} of {} traces fail",
            r.traces
                .iter()
                .filter(|tr| tr.steps.iter().any(|v| !v.passed))
                .count(),
            r.traces.len()
        ),
    };
    let r = ui.label(RichText::new(&summary).color(if good {
        palette(ui).success
    } else if d.checked() {
        palette(ui).error
    } else {
        palette(ui).muted
    }));
    m.record(ui, "summary", &r);
    m.summary = Some(summary);
    if let Some(Ok(report)) = &report {
        for error in &report.configuration_errors {
            ui.label(RichText::new(error).color(palette(ui).error));
            m.feedback.push(error.clone());
        }
        for tr in report
            .traces
            .iter()
            .filter(|tr| tr.steps.iter().any(|v| !v.passed))
        {
            let response = ui.add(button(&format!("Review: {}", tr.label)));
            m.record(ui, format!("failed-{}", tr.id), &response);
            if response.clicked() {
                d.selected_trace = tr.id.clone();
            }
        }
    } else if let Some(Err(error)) = &report {
        m.feedback.push(error.clone());
    }
    if practice {
        m.guidance_order.push(2);
        lesson_presentation::practice_step(ui, s, 2);
    }
    ui.separator();
    let selected = t
        .traces
        .iter()
        .find(|tr| tr.id == d.selected_trace)
        .unwrap_or(&t.traces[0]);
    controls::form_row(ui, "Request trace", |ui, width| {
        let choice = controls::dropdown(ui, "cache-trace", width)
            .selected_text(&selected.label)
            .show_ui(ui, |ui| {
                for trace in &t.traces {
                    let status = match &report {
                        Some(Ok(r)) => r.traces.iter().find(|tr| tr.id == trace.id).map(|tr| {
                            if tr.steps.iter().all(|s| s.passed) {
                                "Pass"
                            } else {
                                "Needs attention"
                            }
                        }),
                        _ => None,
                    };
                    let name = status.map_or_else(
                        || trace.label.clone(),
                        |status| format!("{} · {status}", trace.label),
                    );
                    let r = ui.selectable_value(&mut d.selected_trace, trace.id.clone(), name);
                    m.record(ui, format!("trace-{}", trace.id), &r);
                    m.choices
                        .push((format!("trace-{}", trace.id), r.rect, ui.clip_rect()));
                }
            });
        m.record(ui, "trace", &choice.response);
    });
    let selected = t
        .traces
        .iter()
        .find(|tr| tr.id == d.selected_trace)
        .unwrap_or(&t.traces[0]);
    let observed = report
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .and_then(|r| r.traces.iter().find(|tr| tr.id == selected.id));
    for (index, (step, need)) in selected.steps.iter().zip(&selected.expected).enumerate() {
        ui.push_id((&selected.id, index), |ui| {
            ui.add_space(controls::GAP);
            let heading = ui.strong(format!(
                "Request {} · {} s after first request · GET {}",
                index + 1,
                step.request_time
                    .saturating_sub(selected.steps[0].request_time),
                step.path
            ));
            m.record(ui, format!("request-{index}"), &heading);
            match &step.origin {
                Some(o) => {
                    ui.label(format!(
                        "Origin if contacted: version {}, incoming Age {} s, transfer {} s",
                        o.etag,
                        o.age,
                        o.response_time.saturating_sub(step.request_time)
                    ));
                }
                None => {
                    ui.label("Supplied origin: unavailable");
                }
            }
            if !step.request_control.is_empty() {
                ui.monospace(format!("Request Cache-Control: {}", step.request_control));
            }
            let actual = observed.and_then(|r| r.steps.get(index));
            let needed = expected(need);
            let actual_text =
                actual.map_or_else(|| "Unchecked".into(), |r| actual_label(&r.actual));
            if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 30. {
                ui.columns(2, |cols| {
                    cols[0].strong("Required");
                    lesson_presentation::prose(&mut cols[0], &needed);
                    cols[1].strong("Actual");
                    lesson_presentation::prose(&mut cols[1], &actual_text);
                });
            } else {
                ui.strong("Required");
                lesson_presentation::prose(ui, &needed);
                ui.strong("Actual");
                lesson_presentation::prose(ui, &actual_text);
            }
            if let Some(result) = actual {
                let message = if result.passed {
                    "Meets requirement"
                } else {
                    &result.feedback
                };
                ui.label(RichText::new(message).color(if result.passed {
                    palette(ui).success
                } else {
                    palette(ui).error
                }));
                m.cases
                    .push((format!("{}-{index}", selected.id), result.passed));
                if !result.passed {
                    m.feedback.push(result.feedback.clone());
                }
                controls::collapsing_header(ui, "Inspect body and cache decision").show(ui, |ui| {
                    lesson_presentation::prose(ui, &result.actual.explanation);
                    if let Some(origin) = &step.origin {
                        ui.monospace(format!(
                            "Supplied clock facts: request {} s; response {} s; Date {} s",
                            step.request_time, origin.response_time, origin.date
                        ));
                    }
                    ui.monospace(format!(
                        "Returned Age: {}",
                        result
                            .actual
                            .age
                            .map_or_else(|| "none".into(), |age| format!("{age} s"))
                    ));
                    ui.add(
                        egui::Label::new(
                            RichText::new(
                                result.actual.body.as_deref().unwrap_or("No response body"),
                            )
                            .monospace(),
                        )
                        .wrap(),
                    );
                });
            }
        });
    }
}
impl LabApp {
    pub(super) fn http_cache_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let definition =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let State::HttpCache { simulation } = &attempt.state else {
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
        self.metrics.http_cache = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.http_cache.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i, view as u8, challenge, "shared-http-cache"), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Understand shared storage and request reuse before editing response headers.",
                    Vec::<(String, String)>::new(),
                );
                lesson_presentation::worked(
                    ui,
                    "A separate workshop shows the header choices and the request outcomes.",
                );
                lesson_presentation::worked_rows(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (v, key, label) in [
                        (teaching::View::Example, "try", "Try this example"),
                        (teaching::View::Assignment, "start", "Open full assignment"),
                    ] {
                        let r = ui.add_enabled(active, button(label));
                        self.metrics.http_cache.record(ui, key, &r);
                        if r.clicked() {
                            next.view = v;
                            self.lesson_scroll_reset = true;
                        }
                    }
                });
            } else {
                let practice = view == teaching::View::Example;
                let (task, draft) = if practice {
                    (&definition.example, &mut next.practice)
                } else {
                    (&definition.initial, &mut next.assignment)
                };
                if !practice {
                    ui.label(RichText::new("YOUR TASK").small().color(palette(ui).accent));
                    if let Some(p) = lesson_presentation::entry(&source) {
                        ui.heading(&p.goal);
                        for requirement in &p.requirements {
                            lesson_presentation::prose(ui, &format!("• {requirement}"));
                        }
                    } else {
                        lesson_presentation::prose(ui, &source.objective);
                    }
                }
                ui.add_enabled_ui(active, |ui| {
                    configuration(
                        ui,
                        &source,
                        task,
                        draft,
                        practice,
                        &mut self.metrics.http_cache,
                        &mut self.notice,
                    );
                    results(
                        ui,
                        &source,
                        task,
                        draft,
                        practice,
                        &mut self.metrics.http_cache,
                    );
                    if practice {
                        let r = ui.add(button("Reset example only"));
                        self.metrics.http_cache.record(ui, "reset-example", &r);
                        if r.clicked() {
                            match work::Draft::new(task) {
                                Ok(fresh) => *draft = fresh,
                                Err(e) => self.notice = Some(e),
                            }
                        }
                    }
                    let help = controls::collapsing_header(ui, "Hints and explanation · optional")
                        .show(ui, |ui| {
                            let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
                            let r = ui.add_enabled(
                                active && attempt.hints < source.hints.len(),
                                button("Show a hint"),
                            );
                            self.metrics.http_cache.record(ui, "hint", &r);
                            if r.clicked() {
                                attempt.hints += 1;
                                self.dirty = true;
                            }
                            for hint in source.hints.iter().take(attempt.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            controls::collapsing_header(ui, "Why does this work?")
                                .show(ui, |ui| lesson_presentation::prose(ui, &source.explanation));
                            let worked = controls::collapsing_header(
                                ui,
                                "Show worked answer · reveals one header policy",
                            )
                            .show(ui, |ui| {
                                self.metrics.http_cache.worked_answer_visible = true;
                                for r in &task.resources {
                                    ui.monospace(format!(
                                        "{}\nCache-Control: {}",
                                        r.path, r.reference
                                    ));
                                }
                            });
                            self.metrics.http_cache.record(
                                ui,
                                "worked-answer",
                                &worked.header_response,
                            );
                            controls::collapsing_header(
                                ui,
                                "Supported behavior, limits and references",
                            )
                            .show(ui, |ui| {
                                lesson_presentation::prose(ui, work::HELP);
                                lesson_presentation::prose(ui, &source.simplifications);
                                for url in &source.sources {
                                    ui.hyperlink_to(url, url);
                                }
                            });
                        });
                    self.metrics
                        .http_cache
                        .record(ui, "help", &help.header_response);
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::HttpCache {
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

#[cfg(test)]
#[test]
fn unavailable_reply_can_leave_a_cached_entry_without_returning_that_body() {
    let scenarios = pack::current().unwrap();
    let s = scenarios
        .iter()
        .find(|s| s.id == "lpic2-cache-freshness-handover")
        .unwrap();
    let d = work::source(s).unwrap();
    let report = d
        .initial
        .report_headers(&d.initial.reference_headers())
        .unwrap();
    let actual = &report.traces[0].steps[2].actual;
    assert_eq!(actual.delivery, model::Delivery::Unavailable);
    assert!(actual.body.is_none() && actual.stored);
    let text = actual_label(actual);
    assert!(
        text.contains("No usable response")
            && text.contains("no response body")
            && text.contains("cached copy present")
    );
    let requirement = expected(&d.initial.traces[0].expected[2]);
    assert!(!requirement.contains("unrestricted"));
    assert!(requirement.contains("no body"));
}
