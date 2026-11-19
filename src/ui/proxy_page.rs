//! Native proxy policy editor with actual bounded request decisions.
use super::*;
use dlms_learning_lab::{engine::State, proxy_policy::Action as Access, proxy_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
    pub inspection: Option<String>,
    pub cases: Vec<(String, bool)>,
    pub feedback: Vec<String>,
    pub split_layout: bool,
    pub guidance_order: Vec<usize>,
    pub editor_line_height: f32,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn access(a: Access) -> &'static str {
    match a {
        Access::Allow => "Allowed",
        Access::Deny => "Refused",
    }
}
fn policy_feedback(ui: &mut egui::Ui, r: &work::Report, m: &mut Metrics) {
    let failed = r.policies.iter().filter(|p| !p.passed).collect::<Vec<_>>();
    let combined = failed.len() == 2 && failed[0].counterexample == failed[1].counterexample;
    for (i, p) in failed.iter().enumerate() {
        if combined && i == 1 {
            continue;
        }
        let Some(c) = &p.counterexample else {
            continue;
        };
        let label = if combined {
            "Saved and applied policies"
        } else {
            p.label.as_str()
        };
        let message = format!(
            "{label} need{} attention: client {}, method {}, destination {} must be {}, but is {}. Cover the whole required range, not only the named samples.",
            if combined { "" } else { "s" },
            c.request.client,
            c.request.method,
            c.request.destination_port,
            access(c.required),
            access(c.actual)
        );
        let response = ui.label(RichText::new(&message).color(palette(ui).error));
        if combined {
            m.record(ui, "policy-applied", &response);
            m.record(ui, "policy-saved", &response);
        } else {
            m.record(
                ui,
                if p.label == "Applied policy" {
                    "policy-applied"
                } else {
                    "policy-saved"
                },
                &response,
            );
        }
        m.feedback.push(message);
    }
    if r.policies.len() == 2 && !r.policies[0].passed && r.policies[1].passed {
        ui.label("The saved policy meets the task. Choose Apply saved policy, then Check policy.");
    }
}
fn editor(
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
    let label = ui.strong("squid.conf · simulated configuration");
    if practice {
        let r = ui.add(button("Fill worked configuration"));
        m.record(ui, "fill", &r);
        if r.clicked() {
            *notice = d.edit(t.required_policy.clone()).err();
        }
    }
    let mut text = d.editor.clone();
    m.editor_line_height = ui.text_style_height(&egui::TextStyle::Monospace);
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 12. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scrolled) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("proxy-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt("proxy-editor")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(12)
                        .char_limit(8192),
                )
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let r = scrolled.inner.labelled_by(label.id);
    m.controls.push((
        "editor".into(),
        r.rect.intersect(scrolled.inner_rect),
        ui.clip_rect(),
    ));
    m.fields.push((
        "editor".into(),
        r.rect,
        r.id,
        scrolled.inner_rect.intersect(ui.clip_rect()),
    ));
    if r.changed() {
        *notice = d.edit(text).err();
    }
    if practice {
        m.guidance_order.push(1);
        lesson_presentation::practice_step(ui, s, 1);
    }
    ui.horizontal_wrapped(|ui| {
        for (key, title) in [
            ("save", "Save configuration"),
            ("syntax", "Check syntax"),
            ("apply", "Apply saved policy"),
        ] {
            let r = ui.add(button(title));
            m.record(ui, key, &r);
            if r.clicked() {
                *notice = Some(
                    match key {
                        "save" => d.save(t),
                        "syntax" => d.validate_saved(t),
                        _ => d.apply(t),
                    }
                    .unwrap_or_else(|e| e),
                );
            }
        }
    });
    let r = ui.add(primary(ui, "Check policy"));
    m.record(ui, "check", &r);
    if r.clicked() {
        *notice = d.check(t).err();
    }
    let report = if d.checked() { Some(t.report(d)) } else { None };
    let summary = match &report {
        None => "Configuration not checked since editing".into(),
        Some(Ok(r)) if r.passed() => "All request and policy requirements verified".into(),
        Some(Ok(r)) => {
            let count = r.cases.iter().filter(|c| !c.passed).count()
                + r.policies.iter().filter(|p| !p.passed).count()
                + usize::from(r.configuration_error.is_some());
            format!(
                "Needs attention · {count} {}",
                if count == 1 {
                    "requirement remains"
                } else {
                    "requirements remain"
                }
            )
        }
        Some(Err(e)) => e.clone(),
    };
    let good = matches!(&report,Some(Ok(r))if r.passed());
    ui.label(RichText::new(&summary).color(if good {
        palette(ui).success
    } else if d.checked() {
        palette(ui).error
    } else {
        palette(ui).muted
    }));
    if let Some(Ok(r)) = &report
        && let Some(error) = &r.configuration_error
    {
        lesson_presentation::prose(ui, error);
        m.feedback.push(error.clone());
    }
    if let Some(Ok(r)) = &report
        && r.configuration_error.is_none()
    {
        policy_feedback(ui, r, m);
    }
    m.summary = Some(summary);
}
fn inspector(
    ui: &mut egui::Ui,
    s: &pack::Scenario,
    t: &work::Task,
    d: &mut work::Draft,
    practice: bool,
    m: &mut Metrics,
) {
    if practice {
        m.guidance_order.push(2);
        lesson_presentation::practice_step(ui, s, 2);
    }
    ui.strong("Inspect the applied policy");
    let selected = t
        .cases
        .iter()
        .find(|c| c.id == d.selected_case)
        .map(|c| c.label.as_str())
        .unwrap_or("Choose request");
    let r = controls::form_row(ui, "Request", |ui, width| {
        controls::dropdown(ui, "proxy-request", width)
            .selected_text(selected)
            .show_ui(ui, |ui| {
                for c in &t.cases {
                    let r = ui.selectable_value(&mut d.selected_case, c.id.clone(), &c.label);
                    m.record(ui, &format!("request-{}", c.id), &r);
                    m.choices
                        .push((format!("request-{}", c.id), r.rect, ui.clip_rect()));
                }
            })
            .response
    })
    .inner;
    m.record(ui, "request", &r);
    let c = t.cases.iter().find(|c| c.id == d.selected_case).unwrap();
    ui.monospace(format!(
        "{} · {} · destination {}",
        c.request.client, c.request.method, c.request.destination_port
    ));
    ui.label(format!("Required: {}", access(c.expected)));
    let inspection = controls::collapsing_header(ui, "Inspect applied decision").show(ui, |ui| {
        match d.inspect(t) {
            Err(e) => {
                ui.label(&e);
                m.inspection = Some(e);
            }
            Ok(r) => {
                let message = format!(
                    "Current applied decision: {}. {}",
                    access(r.action),
                    r.explanation
                );
                lesson_presentation::prose(ui, &message);
                m.inspection = Some(message);
                for rule in r.rules {
                    ui.label(format!(
                        "Line {} · {} · {}",
                        rule.line,
                        access(rule.action),
                        if rule.matched {
                            "all tests match"
                        } else {
                            "does not match"
                        }
                    ));
                    ui.horizontal_wrapped(|ui| {
                        for term in rule.terms {
                            ui.monospace(format!(
                                "{}{}: {}",
                                if term.negated { "!" } else { "" },
                                term.name,
                                if term.matched {
                                    "matches"
                                } else {
                                    "does not match"
                                }
                            ));
                        }
                    });
                }
            }
        }
    });
    m.record(ui, "inspect", &inspection.header_response);
    let applied = controls::collapsing_header(ui, "Read applied configuration").show(ui, |ui| {
        ui.add(egui::Label::new(RichText::new(d.world.applied()).monospace()).wrap());
    });
    m.record(ui, "applied", &applied.header_response);
    ui.label(
        RichText::new(
            "Inspection reads current simulated rules; it does not check the assignment.",
        )
        .small()
        .color(palette(ui).muted),
    );
}
fn results(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    ui.separator();
    let heading = ui.strong("Required requests and actual check results");
    m.record(ui, "results", &heading);
    let report = if d.checked() { t.report(d).ok() } else { None };
    for c in &t.cases {
        ui.push_id(&c.id, |ui| {
            ui.strong(&c.label);
            ui.horizontal_wrapped(|ui| {
                ui.monospace(c.request.client.to_string());
                ui.monospace(&c.request.method);
                ui.monospace(format!("destination {}", c.request.destination_port));
                ui.label(format!("Required: {}", access(c.expected)));
            });
            if let Some(r) = report
                .as_ref()
                .and_then(|r| r.cases.iter().find(|r| r.id == c.id))
            {
                let msg = format!(
                    "{} · actual: {}{}",
                    if r.passed {
                        "Meets requirement"
                    } else {
                        "Needs attention"
                    },
                    access(r.actual.action),
                    if r.passed {
                        String::new()
                    } else {
                        format!(". {}", r.actual.explanation)
                    }
                );
                ui.label(RichText::new(&msg).color(if r.passed {
                    palette(ui).success
                } else {
                    palette(ui).error
                }));
                m.cases.push((c.id.clone(), r.passed));
                if !r.passed {
                    m.feedback.push(msg);
                }
            } else {
                ui.label(RichText::new("Unchecked").color(palette(ui).muted));
            }
            ui.add_space(controls::GAP);
        });
    }
}
impl LabApp {
    pub(super) fn proxy_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        let def = match work::source(&source).and_then(|d| work::context(a, &source).map(|()| d)) {
            Ok(d) => d,
            Err(e) => {
                self.notice = Some(e);
                return;
            }
        };
        let State::ProxyPolicy { simulation } = &a.state else {
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
        self.metrics.proxy = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.proxy.record(ui, key, &r);
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
        ui.push_id((i, view as u8, challenge, "proxy-access"), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(ui, &source, "Read request facts and repair a proxy access policy.", Vec::<(String,String)>::new());
                lesson_presentation::worked(ui, "A different site demonstrates each test and rule.");
                lesson_presentation::worked_rows(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (v,key,label) in [(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")] {
                        let r=ui.add_enabled(active,button(label)); self.metrics.proxy.record(ui,key,&r);
                        if r.clicked() { next.view=v; self.lesson_scroll_reset=true; }
                    }
                });
            } else {
                let practice=view==teaching::View::Example;
                let (t,d)=if practice {(&def.example,&mut next.practice)} else {(&def.initial,&mut next.assignment)};
                if !practice {
                    ui.label(RichText::new("YOUR TASK").small().color(palette(ui).accent));
                    if let Some(e)=lesson_presentation::entry(&source) {
                        ui.heading(&e.goal);
                        for requirement in &e.requirements { lesson_presentation::prose(ui,&format!("• {requirement}")); }
                    } else { lesson_presentation::prose(ui,&source.objective); }
                }
                ui.add_enabled_ui(active, |ui| {
                    let editor_min=ui.text_style_height(&egui::TextStyle::Monospace)*21.+2.*f32::from(controls::PAD_X);
                    let request_min=controls::form_label_width(ui)+ui.text_style_height(&egui::TextStyle::Body)*6.+2.*f32::from(controls::PAD_X)+controls::GAP;
                    self.metrics.proxy.split_layout=ui.available_width()>=editor_min+request_min+controls::GAP;
                    if self.metrics.proxy.split_layout {
                        ui.columns(2,|cols| {
                            editor(&mut cols[0],&source,t,d,practice,&mut self.metrics.proxy,&mut self.notice);
                            inspector(&mut cols[1],&source,t,d,practice,&mut self.metrics.proxy);
                        });
                    } else {
                        editor(ui,&source,t,d,practice,&mut self.metrics.proxy,&mut self.notice);
                        ui.separator();
                        inspector(ui,&source,t,d,practice,&mut self.metrics.proxy);
                    }
                    results(ui,t,d,&mut self.metrics.proxy);
                    if practice {
                        let r=ui.add(button("Reset example only")); self.metrics.proxy.record(ui,"reset-example",&r);
                        if r.clicked() { match work::Draft::new(t) { Ok(fresh)=>*d=fresh, Err(e)=>self.notice=Some(e) } }
                    }
                    let help=controls::collapsing_header(ui,"Hints and explanation · optional").show(ui, |ui| {
                        let a=self.progress.attempts.get_mut(&source.id).unwrap();
                        let r=ui.add_enabled(active&&a.hints<source.hints.len(),button("Show a hint"));
                        self.metrics.proxy.record(ui,"hint",&r);
                        if r.clicked() { a.hints+=1; self.dirty=true; }
                        for hint in source.hints.iter().take(a.hints) { lesson_presentation::prose(ui,hint); }
                        let why=controls::collapsing_header(ui,"Why does this work?").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
                        self.metrics.proxy.record(ui,"why",&why.header_response);
                        let worked=controls::collapsing_header(ui,"Show worked answer · reveals one policy").show(ui,|ui| {
                            self.metrics.proxy.worked_answer_visible=true;
                            ui.add(egui::Label::new(RichText::new(&t.required_policy).monospace()).wrap());
                        });
                        self.metrics.proxy.record(ui,"worked-answer",&worked.header_response);
                        let scope=controls::collapsing_header(ui,"Supported syntax, limits and references").show(ui,|ui| {
                            lesson_presentation::prose(ui,work::HELP);
                            lesson_presentation::prose(ui,&source.simplifications);
                            ui.label("No access rules means refusal. If rules exist but none matches, the fallback reverses the last rule's action. An explicit final deny all makes that decision clear.");
                            for url in &source.sources { ui.hyperlink_to(url,url); }
                        });
                        self.metrics.proxy.record(ui,"scope",&scope.header_response);
                    });
                    self.metrics.proxy.record(ui,"help",&help.header_response);
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::ProxyPolicy {
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
