//! Focused native policy editing and actual virtual IP-path trials.
use super::*;
use dlms_learning_lab::{management_policy as policy, management_workspace as work};

#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id, egui::Rect)>,
    pub editor_frame: Option<egui::Rect>,
    pub output: Option<egui::Rect>,
    pub side_by_side: bool,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn diagnostic(error: work::Error) -> String {
    match error {
        work::Error::Document(d) => format!(
            "{}: {d}",
            match d.category {
                dlms_learning_lab::acl_editor::Category::Syntax => "Syntax error",
                dlms_learning_lab::acl_editor::Category::Unsupported => "Unsupported setting",
                dlms_learning_lab::acl_editor::Category::Resolution => "Missing reference",
                dlms_learning_lab::acl_editor::Category::Limit => "Document limit",
                _ => "Policy could not be applied",
            }
        ),
        work::Error::Source(s) => s,
    }
}
fn editor(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    focus: &mut bool,
    notice: &mut Option<String>,
    lesson: (&Scenario, bool),
) {
    let (source, practice) = lesson;
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    ui.strong("Management policy document");
    let mut draft = s.draft.clone();
    let height = ui.text_style_height(&egui::TextStyle::Monospace).ceil() * 14.
        + 2. * f32::from(controls::PAD_Y);
    let width = ui.available_width();
    let mut clip = egui::Rect::NOTHING;
    let (viewport, r) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("management-editor-scroll")
                .auto_shrink([false, false])
                .max_height(height)
                .max_width(width),
            |ui| {
                clip = ui.clip_rect();
                ui.add(
                    egui::TextEdit::multiline(&mut draft)
                        .code_editor()
                        .lock_focus(false)
                        .id(egui::Id::new(("management-policy-document", &t.id)))
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .char_limit(dlms_learning_lab::acl_editor::MAX_TEXT)
                        .desired_rows(14)
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                        ),
                )
            },
        )
        .inner
    });
    // Retain the stable full outer extent for reveal/scroll. Intersecting it
    // with the window here makes a below-fold target move by only a few pixels.
    m.editor = Some((viewport, r.id, clip));
    m.editor_frame = Some(r.rect);
    if ui.is_enabled() && *focus {
        r.request_focus();
        *focus = false;
    }
    if r.changed() {
        s.invalidate();
        *notice = s.edit(draft).err();
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Apply policy"));
        record(m, "apply", &r, ui);
        if r.clicked() {
            *notice = s.apply(t).err().map(diagnostic);
        }
        let r = ui.add(primary(ui, "Check configuration"));
        record(m, "check", &r, ui);
        if r.clicked() {
            *notice = s.check(t).err().map(diagnostic);
        }
    });
    if let Some(e) = notice.as_deref() {
        lesson_presentation::rich(ui, RichText::new(e).color(palette(ui).error));
    }
    lesson_presentation::prose(
        ui,
        if s.draft != s.applied {
            "Edited · apply before inspecting, or check the current document."
        } else if s.checked {
            "Checked against every required client and supplied line."
        } else {
            "Not checked yet."
        },
    );
    results(ui, s, t, m);
}
fn protocol_name(p: policy::Protocol) -> &'static str {
    match p {
        policy::Protocol::Ssh1 => "SSH version 1",
        policy::Protocol::Ssh2 => "SSH version 2",
        policy::Protocol::Telnet => "Telnet",
    }
}
fn trial(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
    source: &Scenario,
    practice: bool,
) {
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    ui.strong("Inspect a supplied trial");
    let mut case = usize::from(s.selected_case);
    let mut line = s.selected_line;
    controls::form_row(ui, "Client trial", |ui, w| {
        let r = controls::dropdown(ui, "management-case", w)
            .selected_text(&t.cases[case].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut case, i, &c.label);
                    record(m, format!("case-{i}"), &r, ui);
                }
            });
        record(m, "case", &r.response, ui);
    });
    controls::form_row(ui, "Terminal line", |ui, w| {
        let r = controls::dropdown(ui, "management-line", w)
            .selected_text(format!("VTY {line}"))
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for l in &t.supplied.lines {
                    let r = ui.selectable_value(&mut line, l.number, format!("VTY {}", l.number));
                    record(m, format!("line-{}", l.number), &r, ui);
                }
            });
        record(m, "line", &r.response, ui);
    });
    if case != usize::from(s.selected_case) || line != s.selected_line {
        *notice = s.select_sample(t, case, line).err();
    }
    let c = &t.cases[case];
    let n = &t.network;
    let client = &n
        .clients
        .iter()
        .find(|v| v.id == c.client)
        .expect("validated client")
        .binding;
    let source_ip = n.topology.devices[&client.device].interfaces[&client.interface]
        .address
        .as_ref()
        .unwrap()
        .address
        .to_string();
    let target_ip = n.topology.devices[&n.management.device].interfaces[&n.management.interface]
        .address
        .as_ref()
        .unwrap()
        .address
        .to_string();
    let caption = match n.topology.verify_endpoint(
        &client.device,
        &n.management.device,
        &n.management.interface,
    ) {
        Ok(p) => format!(
            "Supplied IPv4 route · {}",
            p.forward
                .hops
                .iter()
                .map(|h| h.device.as_str())
                .chain(std::iter::once(n.management.device.as_str()))
                .collect::<Vec<_>>()
                .join(" -> ")
        ),
        Err(_) => "Supplied IPv4 route is unavailable".into(),
    };
    controls::topology_pair(
        ui,
        [&client.device, &source_ip, "Selected client"],
        [&n.management.device, &target_ip, "Management endpoint"],
        palette(ui).accent,
        &caption,
    );
    lesson_presentation::monospace(ui, protocol_name(c.protocol), false);
    let r = ui.add(button("Inspect trial"));
    record(m, "inspect", &r, ui);
    if r.clicked() {
        match s.inspect(t) {
            Ok(_) => *notice = None,
            Err(e) => s.observation = Some(format!("Trial not run: {e}")),
        }
    }
    ui.strong("Trial feedback");
    let height = ui.text_style_height(&egui::TextStyle::Body).ceil() * 8.;
    let output = controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt("management-output")
            .max_height(height)
            .min_scrolled_height(height)
            .auto_shrink([false, false]),
        |ui| {
            if let Some(text) = &s.observation {
                lesson_presentation::monospace(ui, text, true);
            } else {
                lesson_presentation::prose(
                    ui,
                    "No trial observed for the current selection and policy. Apply edits, then Inspect trial.",
                );
            }
        },
    );
    m.output = Some(output.inner_rect);
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &work::Task, m: &mut Metrics) {
    if !s.checked || s.draft != s.applied {
        return;
    }
    let reports = match t.reports(&s.applied) {
        Ok(r) => r,
        Err(e) => {
            lesson_presentation::prose(ui, &e.to_string());
            return;
        }
    };
    let passed = reports.iter().filter(|r| r.passed()).count();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if passed == reports.len() {
            "All management policy requirements match"
        } else {
            "Management policy needs attention"
        })
        .strong()
        .color(if passed == reports.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, reports.len(), r.rect));
    for report in reports {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {} · {}/{} lines",
                if report.passed() {
                    "Matches"
                } else {
                    "Needs work"
                },
                report.label,
                report.matched_lines,
                report.total_lines
            ))
            .strong(),
        );
        if let Some(detail) = report.first_failure {
            lesson_presentation::prose(ui, &detail);
        }
    }
}
impl LabApp {
    pub(super) fn management_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (
            engine::Goal::ManagementPolicy { definition },
            engine::State::ManagementPolicy { simulation },
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
                    record(&mut self.metrics.management, key, &r, ui);
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
                "Builds on IPv4 routes, TCP service ports and standard source access lists.",
                std::iter::empty(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.management, key, &r, ui);
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
            lesson_presentation::prose(
                ui,
                "These trials check the IP path and access settings before login. The lab supplies the server identity; passwords and encryption exchanges are outside this exercise.",
            );
            ui.push_id(("management-workspace",&source.id,practice,challenge),|ui|{
                ui.add_enabled_ui(active,|ui|{
                    self.metrics.management.side_by_side = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 54.;
                    if self.metrics.management.side_by_side {
                        ui.columns(2,|columns|{
                            editor(&mut columns[0],scope,task,&mut self.metrics.management,&mut self.focus_input,&mut self.notice,(&source,practice));
                            trial(&mut columns[1],scope,task,&mut self.metrics.management,&mut self.notice,&source,practice);
                        });
                    }else{
                        editor(ui,scope,task,&mut self.metrics.management,&mut self.focus_input,&mut self.notice,(&source,practice));
                        ui.separator();trial(ui,scope,task,&mut self.metrics.management,&mut self.notice,&source,practice);
                    }
                    if let Some((rect,id,_))=self.metrics.management.editor {self.metrics.editor_viewport=Some(rect);self.metrics.focus=Some(id);}
                    if let Some((_,rect,_))=self.metrics.management.controls.iter().find(|v|v.0=="check") {self.metrics.check=Some(*rect);}
                    if practice{lesson_presentation::practice_step(ui,&source,2);}
                    if practice{
                        let c=controls::collapsing_header(ui,"Worked practice policy · optional").show(ui,|ui|{
                            lesson_presentation::worked_rows(ui,&source);
                            let r=ui.add(button("Fill worked example"));record(&mut self.metrics.management,"fill-example",&r,ui);
                            if r.clicked(){self.notice=scope.edit(task.reference.clone()).err();}
                            lesson_presentation::prose(ui,"This fills the practice editor. Apply or Check when you are ready.");
                        });record(&mut self.metrics.management,"worked-example",&c.header_response,ui);
                        let r=ui.add(button("Reset example"));record(&mut self.metrics.management,"reset-example",&r,ui);
                        if r.clicked(){match work::Scope::new(task){Ok(s)=>{*scope=s;self.notice=None;},Err(e)=>self.notice=Some(e)}}
                    }
                    let c=controls::collapsing_header(ui,"Help and supported settings").show(ui,|ui|{
                        if !practice {
                            let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.management,"hint",&r,ui);
                            if r.clicked(){a.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}
                            controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});
                        }
                        lesson_presentation::prose(ui,"Supported document: explicit SSH version policy; supplied line vty groups; transport input ssh, telnet, both or none; optional access-class NUMBER in; numbered standard source-list sections; and exit. Tab moves to the next control; indentation spaces are optional.");
                        lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}
                    });record(&mut self.metrics.management,"help",&c.header_response,ui);
                });
            });
        }
        if candidate != before && active {
            match candidate.bound(definition) {
                Ok(()) => {
                    let persisted_changed =
                        serde_json::to_value(&candidate).ok() != serde_json::to_value(&before).ok();
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::ManagementPolicy {
                            simulation: Box::new(candidate),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty |= persisted_changed;
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
