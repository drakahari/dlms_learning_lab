//! Virtual client settings or services-file editor, with a read-only lookup terminal.
use super::*;
use dlms_learning_lab::{
    engine::State, service_names as names, service_workspace as work, transport_core::Protocol,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
    pub cases: Vec<(String, bool)>,
    pub feedback: Vec<String>,
    pub editor_line_height: f32,
    pub guidance_order: Vec<usize>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn settings(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let editable = t.mode == work::Mode::Clients;
    let wide = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 34.;
    let label_width = (ui.available_width() * 0.26).clamp(80., 160.);
    let choice_width = controls::button_width(ui, "TCP") + 32.;
    let field_width = if wide {
        (ui.available_width() - label_width - choice_width - 2. * controls::GAP).max(60.)
    } else {
        ui.available_width()
    };
    for s in d.clients.clone() {
        let mut protocol = s.protocol;
        let mut value = s.service.clone();
        let role = &t
            .requests
            .iter()
            .find(|r| r.client == s.id)
            .unwrap()
            .required_role;
        ui.push_id(&s.id, |ui| {
            ui.label(RichText::new(format!("{} · target: {role}", s.id)).strong());
            let row = |ui: &mut egui::Ui| {
                if wide {
                    controls::label(ui, "Transport / service", label_width);
                }
                let r = ui
                    .add_enabled_ui(editable, |ui| {
                        controls::dropdown(ui, "protocol", choice_width)
                            .selected_text(names::protocol_name(protocol).to_uppercase())
                            .show_ui(ui, |ui| {
                                controls::fit_popup_contents(ui);
                                for p in [Protocol::Tcp, Protocol::Udp] {
                                    let r = ui.selectable_value(
                                        &mut protocol,
                                        p,
                                        names::protocol_name(p).to_uppercase(),
                                    );
                                    m.choices.push((
                                        format!("protocol-{}-{}", s.id, names::protocol_name(p)),
                                        r.rect,
                                        controls::popup_content_clip(ui),
                                    ));
                                }
                            })
                            .response
                    })
                    .inner;
                m.record(ui, &format!("protocol-{}", s.id), &r);
                let r = ui.add_enabled(
                    editable,
                    controls::singleline_fitting(ui, &mut value, field_width)
                        .id_salt("service")
                        .hint_text("Service name or decimal port")
                        .char_limit(64),
                );
                m.record(ui, &format!("service-{}", s.id), &r);
                m.fields
                    .push((format!("service-{}", s.id), r.rect, r.id, ui.clip_rect()));
            };
            if wide {
                ui.horizontal(row);
            } else {
                ui.label("Transport / service name or decimal port");
                ui.horizontal_wrapped(row);
            }
        });
        if protocol != s.protocol || value != s.service {
            *notice = d.configure(t, &s.id, protocol, value).err();
        }
        ui.add_space(controls::GAP);
    }
}
fn file_editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let label = ui.strong("/etc/services · virtual file");
    let mut value = d.editor.clone();
    m.editor_line_height = ui.text_style_height(&egui::TextStyle::Monospace);
    let height = m.editor_line_height * 9. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scrolled) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("services-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut value)
                        .id_salt("services-editor")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(9)
                        .char_limit(32768),
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
        *notice = d.edit(t, value).err();
    }
    let r = ui.add(button("Save virtual file"));
    m.record(ui, "save", &r);
    if r.clicked() {
        *notice = Some(
            d.save(t)
                .map(|()| {
                    "Saved. New lookups use this file; server listeners have not changed.".into()
                })
                .unwrap_or_else(|e| e),
        );
    }
}
fn terminal(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let label = ui.strong("Service lookup terminal · simulated");
    m.record(ui, "terminal-output", &label);
    let hint = if d.transcript.is_empty() {
        "Try: getent services\nThe result appears here after Run."
    } else {
        d.transcript.as_str()
    };
    controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt("services-output")
            .max_height(ui.text_style_height(&egui::TextStyle::Monospace) * 9.)
            .min_scrolled_height(ui.text_style_height(&egui::TextStyle::Monospace) * 4.)
            .auto_shrink([false, true])
            .stick_to_bottom(true),
        |ui| {
            ui.add(egui::Label::new(RichText::new(hint).monospace()).extend());
        },
    );
    let mut run = false;
    ui.horizontal_wrapped(|ui| {
        let run_width = controls::button_width(ui, "Run");
        let width = (ui.available_width() - run_width - controls::GAP).max(100.);
        let r = ui.add(
            controls::singleline_fitting(ui, &mut d.terminal_input, width)
                .id_salt("services-command")
                .hint_text("getent services NAME/tcp")
                .char_limit(256),
        );
        m.record(ui, "terminal-input", &r);
        m.fields
            .push(("terminal-input".into(), r.rect, r.id, ui.clip_rect()));
        run |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        let r = ui.add(button("Run"));
        m.record(ui, "run", &r);
        run |= r.clicked();
    });
    if run {
        let input = d.terminal_input.clone();
        *notice = d.command(t, &input).err();
    }
}
fn results(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let report = if d.checked() { t.report(d).ok() } else { None };
    let summary = match &report {
        None => "Not checked since editing".into(),
        Some(r) if r.passed() => "All client and reference requirements verified".into(),
        Some(r) => format!(
            "Needs attention · {} of {} request cases meet the task",
            r.cases.iter().filter(|c| c.passed).count(),
            r.cases.len()
        ),
    };
    m.summary = Some(summary.clone());
    let r = ui.strong(summary);
    m.record(ui, "results", &r);
    if let Some(report) = report {
        if report.unsaved {
            let msg = "Unsaved file changes: choose Save virtual file, then Check clients.";
            ui.label(RichText::new(msg).color(palette(ui).error));
            m.feedback.push(msg.into());
        }
        for e in &report.database_errors {
            ui.label(
                RichText::new(format!("Reference needs attention: {e}")).color(palette(ui).error),
            );
            m.feedback.push(e.clone());
        }
        for c in &report.cases {
            ui.separator();
            ui.strong(format!(
                "{} · {}",
                if c.passed {
                    "Meets requirement"
                } else {
                    "Needs attention"
                },
                c.label
            ));
            ui.label(format!("Required: {}", c.required));
            if let Some(a) = &c.actual {
                ui.label(
                    RichText::new(format!(
                        "Actual: {}/{} · {}",
                        a.resolved.port,
                        names::protocol_name(a.resolved.protocol),
                        match a.transport.outcome {
                            dlms_learning_lab::transport_core::Outcome::Delivered =>
                                "Payload delivered",
                            dlms_learning_lab::transport_core::Outcome::NoListener => "No listener",
                            _ => "Transport incomplete",
                        }
                    ))
                    .monospace(),
                );
            }
            ui.label(&c.explanation);
            m.cases.push((c.id.clone(), c.passed));
            if !c.passed {
                m.feedback.push(c.explanation.clone());
            }
            if let Some(a) = &c.actual {
                controls::collapsing_header(ui, &format!("Transport trace · {}", c.label))
                    .id_salt(&c.id)
                    .show(ui, |ui| {
                        for event in &a.transport.trace {
                            ui.label(RichText::new(format!("{event:?}")).monospace());
                        }
                    });
            }
        }
    }
}
impl LabApp {
    pub(super) fn service_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let State::ServiceClients { simulation } = &a.state else {
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
        self.metrics.services = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, title) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(title).selected(view == v));
                    self.metrics.services.record(ui, key, &r);
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
        ui.push_id((i,view as u8,challenge,"service-workspace"),|ui|{
            if view==teaching::View::Introduction {
                lesson_presentation::concepts(ui,&source,"Distinguish a service name, a transport destination and a running listener.",Vec::<(String,String)>::new());lesson_presentation::worked(ui,"Different example values demonstrate the lookup and the client request.");lesson_presentation::worked_rows(ui,&source);
                ui.horizontal_wrapped(|ui|{for (v,key,title) in [(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")] {let r=ui.add_enabled(active,button(title));self.metrics.services.record(ui,key,&r);if r.clicked(){next.view=v;self.lesson_scroll_reset=true;}}});
            }else{
                let practice=view==teaching::View::Example;let(t,d)=if practice {(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
                if !practice {ui.label(RichText::new("YOUR TASK").small().color(palette(ui).accent));if let Some(e)=lesson_presentation::entry(&source){ui.heading(&e.goal);for r in &e.requirements{lesson_presentation::prose(ui,&format!("• {r}"));}}}
                ui.add_enabled_ui(active,|ui|{
                    let facts=ui.strong(format!("Supplied server · {}",t.profile.server));self.metrics.services.record(ui,"server-facts",&facts);
                    egui::Grid::new("server-listeners").min_row_height(controls::table_height(ui)).spacing([controls::GAP,8.]).show(ui,|ui|{ui.strong("Role");ui.strong("Port / transport");ui.end_row();for l in &t.profile.listeners {let r=ui.label(&l.role);self.metrics.services.record(ui,&format!("listener-role-{}",l.role),&r);let r=ui.monospace(format!("{} / {}",l.port,names::protocol_name(l.protocol)));self.metrics.services.record(ui,&format!("listener-value-{}",l.role),&r);ui.end_row();}});
                    ui.label("These are the exercise's supplied listeners. Test cases also temporarily remove a listener; a name lookup alone cannot detect that outage.");
                    if practice {self.metrics.services.guidance_order.push(0);lesson_presentation::practice_step(ui,&source,0);let r=ui.add(button("Fill worked configuration"));self.metrics.services.record(ui,"fill",&r);if r.clicked(){if t.mode==work::Mode::Database {self.notice=d.edit(t,t.required_database.clone()).err();}else{for c in &t.required_clients {self.notice=d.configure(t,&c.id,c.protocol,c.service.clone()).err();}}}}
                    settings(ui,t,d,&mut self.metrics.services,&mut self.notice);
                    if t.mode==work::Mode::Database {file_editor(ui,t,d,&mut self.metrics.services,&mut self.notice);}
                    if practice {self.metrics.services.guidance_order.push(1);lesson_presentation::practice_step(ui,&source,1);}
                    terminal(ui,t,d,&mut self.metrics.services,&mut self.notice);
                    if practice {self.metrics.services.guidance_order.push(2);lesson_presentation::practice_step(ui,&source,2);}
                    let r=ui.add(primary(ui,"Check clients"));self.metrics.services.record(ui,"check",&r);if r.clicked(){self.notice=d.check(t).err();}
                    results(ui,t,d,&mut self.metrics.services);
                    if practice {let r=ui.add(button("Reset example only"));self.metrics.services.record(ui,"reset-example",&r);if r.clicked(){match work::Draft::new(t){Ok(fresh)=>*d=fresh,Err(e)=>self.notice=Some(e)}}}
                    let help=controls::collapsing_header(ui,"Hints and explanation · optional").show(ui,|ui|{
                        let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add_enabled(active&&a.hints<source.hints.len(),button("Show a hint"));self.metrics.services.record(ui,"hint",&r);if r.clicked(){a.hints+=1;self.dirty=true;}for h in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,h);}
                        controls::collapsing_header(ui,"Why does this work?").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
                        let answer=controls::collapsing_header(ui,"Show worked answer · reveals one configuration").show(ui,|ui|{self.metrics.services.worked_answer_visible=true;if t.mode==work::Mode::Database{ui.add(egui::Label::new(RichText::new(&t.required_database).monospace()).wrap());}else{for s in &t.required_clients{ui.monospace(format!("{}: {} · {}",s.id,names::protocol_name(s.protocol),s.service));}}});self.metrics.services.record(ui,"worked-answer",&answer.header_response);
                        let scope=controls::collapsing_header(ui,"Supported commands, limits and references").show(ui,|ui|{lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}});self.metrics.services.record(ui,"scope",&scope.header_response);
                    });self.metrics.services.record(ui,"help",&help.header_response);
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::ServiceClients {
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
