//! Source-bound mail editors and bounded command practice, never a host MTA.
use super::*;
use dlms_learning_lab::{smtp_config::Config, smtp_source as binding, smtp_workspace as work};
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
fn apply(s: &mut work::Scope, t: &work::Task, a: &work::Action, notice: &mut Option<String>) {
    if let Err(e) = s.apply(t, a) {
        *notice = Some(e);
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
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    controls::form_row(ui, "Mail server", |ui, width| {
        let response = controls::dropdown(ui, "smtp-server", width)
            .selected_text(&s.selected)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let start = m.controls.len();
                for (id, node) in &t.facts.world.nodes {
                    let r = ui.selectable_value(
                        &mut s.selected,
                        id.clone(),
                        format!("{id} · {}", node.address),
                    );
                    m.record(ui, &format!("server:{id}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for r in &mut m.controls[start..] {
                    r.2 = clip;
                }
            });
        controls::retain_dropdown_focus(ui, &response.response);
        m.record(ui, "server", &response.response);
    });
    let node = &t.facts.world.nodes[&s.selected];
    lesson_presentation::monospace(
        ui,
        &format!("{} · {}:{}", node.hostname, node.address, node.port),
        false,
    );
    let label = ui.strong("/etc/postfix/main.cf · simulated file");
    let mut text = s.drafts[&s.selected].clone();
    let line = ui.text_style_height(&egui::TextStyle::Monospace);
    let height = line * 10. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scroll) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("smtp-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt(("smtp-editor", &s.selected))
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(10)
                        .char_limit(8192),
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
    if r.changed() {
        apply(
            s,
            t,
            &work::Action::Edit {
                node: s.selected.clone(),
                text,
            },
            notice,
        );
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save configuration"));
        m.record(ui, "save", &r);
        if r.clicked() {
            apply(
                s,
                t,
                &work::Action::Save {
                    node: s.selected.clone(),
                },
                notice,
            );
        }
        let r = ui.add(button("Reload saved settings"));
        m.record(ui, "reload", &r);
        if r.clicked() {
            apply(
                s,
                t,
                &work::Action::Run {
                    input: "postfix reload".into(),
                },
                notice,
            );
        }
    });
    let saved = s.drafts[&s.selected] == s.runtime.files[&s.selected];
    lesson_presentation::prose(
        ui,
        if !saved {
            "Editor changes not saved. Reload uses the saved file, not this draft."
        } else if s.runtime.files[&s.selected] != s.runtime.active[&s.selected] {
            "Saved · running settings differ. Reload after correcting the file."
        } else {
            "Saved · running settings match this file."
        },
    );
    if let Some(fields) = t.editable.get(&s.selected) {
        lesson_presentation::prose(
            ui,
            &format!(
                "Task settings: {}. Preserve the behavior of other supplied settings.",
                fields.iter().cloned().collect::<Vec<_>>().join(", ")
            ),
        );
    } else {
        lesson_presentation::prose(
            ui,
            "Inspect this server; preserve its supplied settings for this task.",
        );
    }
}
fn terminal(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    phase: (bool, &mut bool),
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let (practice, focus) = phase;
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    ui.strong(format!("Command practice · {}", s.selected));
    if practice {
        let commands = source
            .guide
            .iter()
            .flat_map(|g| g.commands.iter())
            .filter(|c| !c.simulator_only)
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            let key = egui::Id::new(("smtp-command-choice", &source.id));
            let mut n = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, width| {
                let r = controls::dropdown(ui, "smtp-example-command", width)
                    .selected_text(&commands[n].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let start = m.controls.len();
                        for (at, c) in commands.iter().enumerate() {
                            let r = ui.selectable_value(&mut n, at, &c.command);
                            m.record(ui, &format!("example-command:{at}"), &r);
                        }
                        let clip = controls::popup_content_clip(ui);
                        for r in &mut m.controls[start..] {
                            r.2 = clip;
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "example-command", &r.response);
            });
            lesson_presentation::prose(ui, &commands[n].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                s.input = commands[n].command.clone();
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, n));
        }
    }
    let height = (ui.text_style_height(&egui::TextStyle::Monospace) * 9.).max(140.);
    egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
        let r=ui.label("Actual command results");m.record(ui,"output-title",&r);
        controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("smtp-output").auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{
            if s.output.is_empty(){lesson_presentation::prose(ui,"No command has run. Type a supported command below, then press Enter or Run. Simulator observations are labeled separately.");}
            else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}
        });
    });
    let mut run = false;
    let mut input = |ui: &mut egui::Ui, run: &mut bool, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("smtp-command", &source.id, practice)),
            4096,
            ui.is_enabled(),
            "Command · Enter to run",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            s.input = text;
        }
        if *focus && ui.is_enabled() {
            r.response.request_focus();
            *focus = false;
        }
        *run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Use one command within4KiB. The existing draft was retained.",
            );
        }
    };
    let width = controls::button_width(ui, "Run");
    if ui.available_width()
        > width + ui.text_style_height(&egui::TextStyle::Body) * 8. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let field = (ui.available_width() - width - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(field, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| input(ui, &mut run, m),
            );
            let r = ui.add(button("Run"));
            m.record(ui, "run", &r);
            run |= r.clicked();
        });
    } else {
        input(ui, &mut run, m);
        let r = ui.add(button("Run"));
        m.record(ui, "run", &r);
        run |= r.clicked();
    }
    if run && !s.input.trim().is_empty() {
        let command = s.input.clone();
        apply(s, t, &work::Action::Run { input: command }, notice);
        s.input.clear();
        *focus = true;
    }
    controls::collapsing_header(ui, "Supported commands and model differences").show(ui, |ui| {
        lesson_presentation::prose(ui, dlms_learning_lab::smtp_console::HELP);
    });
}
fn mail(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.strong("Sample mail · actual retained state");
    for (id, sample) in &t.facts.samples {
        let status = match &s.runtime.messages[id].status {
            dlms_learning_lab::smtp_queue::Status::Queued => "Queued · no attempt yet".into(),
            dlms_learning_lab::smtp_queue::Status::Deferred { reason } => {
                format!("Deferred · {reason}")
            }
            dlms_learning_lab::smtp_queue::Status::Failed { reason } => {
                format!("Failed · {reason}")
            }
            dlms_learning_lab::smtp_queue::Status::Delivered { node, mailboxes } => format!(
                "Delivered on {node} to {}",
                mailboxes.iter().cloned().collect::<Vec<_>>().join(", ")
            ),
            dlms_learning_lab::smtp_queue::Status::Partial {
                node,
                mailboxes,
                failures,
            } => format!(
                "Partial delivery on {node}: {} mailboxes, {} failures",
                mailboxes.len(),
                failures.len()
            ),
        };
        controls::form_row(ui, &sample.subject, |ui, _| {
            lesson_presentation::monospace(ui, &sample.recipient, false);
        });
        lesson_presentation::prose(ui, &format!("Queue owner: {} · {status}", sample.entry));
    }
    let response=controls::collapsing_header(ui,"Replay sample mail · simulator helper").show(ui,|ui|{
        lesson_presentation::prose(ui,"Restore only this practice or assignment's original sample mail to Queued. Saved and running configurations, editor drafts and console history stay. This is not a Linux/Postfix command. Failed mail is not silently recovered by changing a route.");
        let r=ui.add(button("Reset sample mail only"));m.record(ui,"reset-samples",&r);if r.clicked(){apply(s,t,&work::Action::ResetSamples,notice);}
    });
    m.record(ui, "replay-help", &response.header_response);
}
fn results(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &work::Scope,
    source: &Scenario,
    practice: bool,
    m: &mut Metrics,
) {
    if !s.checked {
        let r = lesson_presentation::prose(
            ui,
            "Unchecked · Check configuration tests current saved/running routes, delivery records and required relay refusals.",
        );
        m.record(ui, "unchecked", &r);
        return;
    }
    let rows = s.checks(t);
    let failed = rows.iter().filter(|r| !r.passed).count();
    let r = ui.strong(if failed > 0 {
        "Configuration needs attention"
    } else if practice {
        "Example checks pass"
    } else {
        "Assignment complete"
    });
    m.record(ui, "result-title", &r);
    m.result = Some((rows.len(), failed, r.rect));
    lesson_presentation::prose(
        ui,
        &format!(
            "{}/{} required outcomes pass.",
            rows.len() - failed,
            rows.len()
        ),
    );
    for row in rows.iter().filter(|r| !r.passed) {
        let r = lesson_presentation::prose(
            ui,
            &format!("Needs attention · {}\n{}", row.label, row.detail),
        );
        m.record(ui, "failed-case", &r);
    }
    controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
        for row in rows.iter().filter(|r| r.passed) {
            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", row.label, row.detail));
        }
    });
    if failed == 0 {
        lesson_presentation::prose(ui, &source.explanation);
    }
}
impl LabApp {
    pub(super) fn smtp_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        if let Err(e) = binding::context(original, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (engine::Goal::SmtpRouting { definition }, engine::State::SmtpRouting { simulation }) =
            (&source.goal, &original.state)
        else {
            return;
        };
        let first = egui::Id::new(("smtp-first-frame", &source.id, original.visited));
        if !ui
            .ctx()
            .data(|d| d.get_temp::<bool>(first))
            .unwrap_or(false)
        {
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
                    self.metrics.smtp.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = false;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            lesson_presentation::worked(
                ui,
                "Separate mail example · follow the saved file, running settings and actual recipient.",
            );
            let r = ui.scope(|ui| lesson_presentation::worked_rows(ui, &source));
            self.metrics.smtp.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.smtp.record(ui, "try", &r);
            self.metrics.example_action = Some(r.rect);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
                self.focus_input = false;
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
            ui.push_id(("smtp-workspace", &source.id, practice), |ui| {
                ui.add_enabled_ui(active, |ui| {
                    if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 45. {
                        ui.columns(2, |cols| {
                            editor(
                                &mut cols[0],
                                t,
                                s,
                                &source,
                                practice,
                                &mut self.metrics.smtp,
                                &mut self.notice,
                            );
                            terminal(
                                &mut cols[1],
                                t,
                                s,
                                &source,
                                (practice, &mut self.focus_input),
                                &mut self.metrics.smtp,
                                &mut self.notice,
                            );
                        });
                    } else {
                        editor(
                            ui,
                            t,
                            s,
                            &source,
                            practice,
                            &mut self.metrics.smtp,
                            &mut self.notice,
                        );
                        ui.add_space(controls::GAP);
                        terminal(
                            ui,
                            t,
                            s,
                            &source,
                            (practice, &mut self.focus_input),
                            &mut self.metrics.smtp,
                            &mut self.notice,
                        );
                    }
                    ui.separator();
                    mail(ui, t, s, &mut self.metrics.smtp, &mut self.notice);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(primary(ui, "Check configuration"));
                        self.metrics.check = Some(r.rect);
                        self.metrics.smtp.record(ui, "check", &r);
                        if r.clicked() {
                            apply(s, t, &work::Action::Check, &mut self.notice);
                        }
                        if practice {
                            let r = ui.add(button("Reset example only"));
                            self.metrics.smtp.record(ui, "reset-example", &r);
                            if r.clicked() {
                                match work::Session::new(definition) {
                                    Ok(fresh) => *s = fresh.practice,
                                    Err(e) => self.notice = Some(e),
                                }
                            }
                        }
                    });
                    results(ui, t, s, &source, practice, &mut self.metrics.smtp);
                });
            });
            controls::collapsing_header(ui,"Optional help and simulation scope").show(ui,|ui|{
                if !practice{
                    let a=self.progress.attempts.get_mut(&source.id).unwrap();if ui.add_enabled(active&&a.hints<source.hints.len(),button("Show a hint")).clicked(){a.hints+=1;self.dirty=true;}
                    for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}
                    controls::collapsing_header(ui,"Worked approach · reveals the assignment").show(ui,|ui|{
                        lesson_presentation::prose(ui,&source.explanation);
                        for a in &definition.reference{match a{
                            work::Action::Edit{node,text}=>{lesson_presentation::prose(ui,&format!("On {node}, edit the supplied file:"));lesson_presentation::monospace(ui,text,true);},
                            work::Action::Save{node}=>{lesson_presentation::prose(ui,&format!("Select {node} and Save configuration."));},
                            work::Action::Reload{node}=>{lesson_presentation::prose(ui,&format!("Select {node} and run postfix reload."));},
                            work::Action::Flush{node}=>{lesson_presentation::prose(ui,&format!("Select {node} and run postqueue -f; inspect the actual sample states."));},
                            work::Action::Run{input}=>{lesson_presentation::monospace(ui,input,false);},
                            work::Action::ResetSamples=>{lesson_presentation::prose(ui,"Use Reset sample mail only if replaying a failed sample.");},
                            work::Action::Check=>{lesson_presentation::prose(ui,"Check configuration verifies delivery and refusal outcomes together.");},
                        }}
                    });
                }
                controls::collapsing_header(ui,"Supplied servers and local mailboxes").show(ui,|ui|{for(id,n)in &t.facts.world.nodes{
                    lesson_presentation::monospace(ui,&format!("{id}: {} · {} · local users {}",n.hostname,n.address,n.users.iter().cloned().collect::<Vec<_>>().join(", ")),true);
                    if let Ok(c)=Config::parse(&s.runtime.active[id]){lesson_presentation::prose(ui,&format!("Running local domains: {} · next hop: {}",c.destinations.into_iter().collect::<Vec<_>>().join(", "),c.relay.map(|r|format!("{}:{}",r.host,r.port)).unwrap_or_else(||"supplied literal destination route, if present".into())));}
                }});
                lesson_presentation::prose(ui,&source.simplifications);for r in &source.sources{ui.hyperlink_to(r,r);}
            });
        }
        if active && next != before {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::SmtpRouting {
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
