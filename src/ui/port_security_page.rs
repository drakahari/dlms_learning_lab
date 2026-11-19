//! Native secure-port settings, metadata replay and specific ordered outcomes.
use super::*;
use dlms_learning_lab::{port_security as core, port_security_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub scope: String,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn status(p: &core::Port) -> &'static str {
    if !p.admin_up {
        "Administratively down"
    } else if p.error_disabled {
        "Error-disabled"
    } else {
        "Up"
    }
}
fn field(
    ui: &mut egui::Ui,
    text: &mut String,
    key: &str,
    hint: &str,
    width: f32,
    m: &mut Metrics,
    focus: &mut bool,
) -> bool {
    let r = ui.add(
        controls::singleline_fitting(ui, text, width)
            .id(egui::Id::new(("secure-port-field", &m.scope, key)))
            .hint_text(hint),
    );
    if *focus {
        r.request_focus();
        *focus = false;
    }
    m.fields.push((key.into(), r.rect, r.id, ui.clip_rect()));
    r.changed()
}
fn mode(
    ui: &mut egui::Ui,
    id: &str,
    value: &mut core::ViolationMode,
    width: f32,
    m: &mut Metrics,
) -> bool {
    let before = *value;
    let r = controls::dropdown(ui, id, width)
        .selected_text(format!("{value:?}"))
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for v in [
                core::ViolationMode::Protect,
                core::ViolationMode::Restrict,
                core::ViolationMode::Shutdown,
            ] {
                let r = ui.selectable_value(value, v, format!("{v:?}"));
                record(m, &format!("{id}-{v:?}"), &r, ui);
            }
        });
    record(m, id, &r.response, ui);
    before != *value
}
fn topology(ui: &mut egui::Ui, w: &core::World) {
    let state = w.forwarding();
    let fabric = state.fabric();
    let switch = w.switch();
    let mut cards = vec![];
    for p in w.ports() {
        use dlms_learning_lab::switching::Jack;
        let jack = Jack::Switch {
            switch: switch.into(),
            port: p.name.clone(),
        };
        let host = fabric
            .cables
            .iter()
            .find_map(|c| match (&c.a, &c.b) {
                (Jack::Host { host }, other) | (other, Jack::Host { host }) if *other == jack => {
                    Some(host)
                }
                _ => None,
            })
            .unwrap();
        cards.push([
            fabric.hosts[host].label.clone(),
            state.address(host).unwrap().text(),
            format!("{} · {}", p.name, status(p)),
        ]);
    }
    let refs = cards
        .iter()
        .map(|c| [c[0].as_str(), c[1].as_str(), c[2].as_str()])
        .collect::<Vec<_>>();
    let values = fabric.switches[switch]
        .vlans
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let vlans = format!(
        "{} {values}",
        if fabric.switches[switch].vlans.len() == 1 {
            "VLAN"
        } else {
            "VLANs"
        }
    );
    if !controls::topology_star(
        ui,
        [&fabric.switches[switch].label, switch, &vlans],
        &refs,
        "Each workstation has its own attachment; replay uses the attachment names below.",
    ) {
        lesson_presentation::rich(
            ui,
            RichText::new(format!("{} · {vlans}", fabric.switches[switch].label)).strong(),
        );
        for c in cards {
            lesson_presentation::prose(ui, &c.join(" · "));
        }
    }
}
fn settings(
    ui: &mut egui::Ui,
    scope: &mut work::Scope,
    m: &mut Metrics,
    focus: &mut bool,
    notice: &mut Option<String>,
) {
    lesson_presentation::rich(ui, RichText::new("Port settings").strong());
    let h = ui.text_style_height(&egui::TextStyle::Body);
    let widths = [
        h * 7.,
        h * 4.3,
        h * 4.5,
        h * 8.,
        h * 21.,
        controls::button_width(ui, "Shut down"),
    ];
    let wide =
        ui.available_width() >= widths.iter().sum::<f32>() + ui.spacing().item_spacing.x * 5.;
    let mut administrative = None;
    let before = scope.drafts.clone();
    if wide {
        egui::Grid::new("secure-settings-grid")
            .spacing(egui::vec2(12., 10.))
            .show(ui, |ui| {
                for (label, width) in [
                    "Port / current state",
                    "Security",
                    "Maximum",
                    "Violation mode",
                    "Configured MAC addresses",
                    "Recovery",
                ]
                .into_iter()
                .zip(widths)
                {
                    controls::label(ui, RichText::new(label).strong(), width);
                }
                ui.end_row();
                for d in &mut scope.drafts {
                    let p = scope.world.port(&d.port.0).unwrap();
                    controls::label(ui, format!("{}\n{}", d.port.0, status(p)), widths[0]);
                    controls::cell(ui, widths[1], |ui| {
                        let r = ui.checkbox(&mut d.enabled, "Enabled");
                        record(m, &format!("enable-{}", d.port.0), &r, ui);
                    });
                    controls::cell(ui, widths[2], |ui| {
                        field(
                            ui,
                            &mut d.maximum,
                            &format!("maximum-{}", d.port.0),
                            "1–16",
                            widths[2],
                            m,
                            focus,
                        );
                    });
                    controls::cell(ui, widths[3], |ui| {
                        mode(ui, &format!("mode-{}", d.port.0), &mut d.mode, widths[3], m);
                    });
                    controls::cell(ui, widths[4], |ui| {
                        field(
                            ui,
                            &mut d.configured,
                            &format!("configured-{}", d.port.0),
                            "MACs separated by commas",
                            widths[4],
                            m,
                            &mut false,
                        );
                    });
                    controls::cell(ui, widths[5], |ui| {
                        let label = if p.admin_up { "Shut down" } else { "Bring up" };
                        let r = ui.add(button(label));
                        record(m, &format!("admin-{}", p.name), &r, ui);
                        if r.clicked() {
                            administrative = Some((p.name.clone(), !p.admin_up));
                        }
                    });
                    ui.end_row();
                }
            });
    } else {
        for d in &mut scope.drafts {
            let p = scope.world.port(&d.port.0).unwrap();
            ui.separator();
            lesson_presentation::rich(
                ui,
                RichText::new(format!("{} · {}", d.port.0, status(p))).strong(),
            );
            ui.horizontal_wrapped(|ui| {
                let r = ui.checkbox(&mut d.enabled, "Security enabled");
                record(m, &format!("enable-{}", d.port.0), &r, ui);
                let r = ui.add(button(if p.admin_up { "Shut down" } else { "Bring up" }));
                record(m, &format!("admin-{}", p.name), &r, ui);
                if r.clicked() {
                    administrative = Some((p.name.clone(), !p.admin_up));
                }
            });
            controls::form_row(ui, "Maximum", |ui, w| {
                field(
                    ui,
                    &mut d.maximum,
                    &format!("maximum-{}", d.port.0),
                    "1–16",
                    w.min(h * 6.),
                    m,
                    focus,
                );
            });
            controls::form_row(ui, "Violation mode", |ui, w| {
                mode(ui, &format!("mode-{}", d.port.0), &mut d.mode, w, m);
            });
            controls::form_row(ui, "Configured MAC addresses", |ui, w| {
                field(
                    ui,
                    &mut d.configured,
                    &format!("configured-{}", d.port.0),
                    "MACs separated by commas",
                    w,
                    m,
                    &mut false,
                );
            });
        }
    }
    if before != scope.drafts {
        scope.checked = false;
    }
    if let Some((port, up)) = administrative {
        *notice = scope.admin(&port, up).err();
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Apply settings"));
        record(m, "apply", &r, ui);
        if r.clicked() {
            *notice = scope.apply().err();
        }
        ui.label(if scope.applied() {
            "Current settings applied"
        } else {
            "Pending settings"
        });
    });
    if let Some(e) = notice.as_deref() {
        lesson_presentation::rich(ui, RichText::new(e).color(palette(ui).error));
    }
}
fn secure_table(ui: &mut egui::Ui, scope: &work::Scope) {
    let rows = scope.world.secure_rows();
    lesson_presentation::rich(ui, RichText::new("Current ports").strong());
    for p in scope.world.ports() {
        lesson_presentation::prose(
            ui,
            &format!(
                "{} · {} · {} secure address(es) · violations {}{}",
                p.name,
                status(p),
                p.count(),
                p.violations,
                if p.violations == core::MAX_COUNTER {
                    " (counter saturated)"
                } else {
                    ""
                }
            ),
        );
    }
    controls::collapsing_header(ui, "Secure address table · current entries").show(ui, |ui| {
        for r in rows {
            lesson_presentation::monospace(
                ui,
                &format!(
                    "{} · VLAN {} · {} · {}",
                    r.port,
                    r.vlan,
                    r.address.text(),
                    match r.origin {
                        core::Origin::Configured => "Configured",
                        core::Origin::DynamicSecure => "Learned securely",
                    }
                ),
                true,
            );
        }
        if scope.world.secure_rows().is_empty() {
            ui.label("No secure addresses yet");
        }
    });
}
fn replay(ui: &mut egui::Ui, scope: &mut work::Scope, m: &mut Metrics) {
    let before = (
        scope.host.clone(),
        scope.source.clone(),
        scope.destination.clone(),
    );
    lesson_presentation::rich(ui, RichText::new("Replay one frame").strong());
    controls::form_row(ui, "Attachment", |ui, w| {
        let r = controls::dropdown(ui, "secure-attachment", w)
            .selected_text(&scope.host.0)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for h in scope.world.baseline().fabric().hosts.keys() {
                    let r = ui.selectable_value(&mut scope.host.0, h.clone(), h);
                    record(m, &format!("attachment-{h}"), &r, ui);
                }
            });
        record(m, "attachment", &r.response, ui);
    });
    controls::form_row(ui, "Source MAC", |ui, w| {
        field(
            ui,
            &mut scope.source,
            "source",
            "Six hexadecimal pairs",
            w.min(390.),
            m,
            &mut false,
        );
    });
    controls::form_row(ui, "Destination MAC", |ui, w| {
        field(
            ui,
            &mut scope.destination,
            "destination",
            "Individual or broadcast MAC",
            w.min(390.),
            m,
            &mut false,
        );
    });
    if before
        != (
            scope.host.clone(),
            scope.source.clone(),
            scope.destination.clone(),
        )
    {
        scope.checked = false;
    }
    let r = ui.add(button("Send frame"));
    record(m, "send", &r, ui);
    if r.clicked() {
        let _ = scope.send();
    }
    let height = ui.text_style_height(&egui::TextStyle::Body) * 5.;
    controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt("secure-replay-output")
            .max_height(height)
            .min_scrolled_height(height)
            .stick_to_bottom(true),
        |ui| {
            if scope.output.is_empty() {
                lesson_presentation::prose(
                    ui,
                    "Actual replay results appear here after Send frame.",
                );
            }
            for l in &scope.output {
                lesson_presentation::rich(
                    ui,
                    RichText::new(&l.text).color(if l.error {
                        palette(ui).error
                    } else {
                        palette(ui).text
                    }),
                );
            }
        },
    );
}
impl LabApp {
    pub(super) fn port_security_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::PortSecurity { definition }, engine::State::PortSecurity { simulation }) =
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
                    record(&mut self.metrics.port_security, key, &r, ui);
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
                "Builds on VLAN membership and Ethernet source learning.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(ui, &definition.example.instructions);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.port_security, key, &r, ui);
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
            self.metrics.port_security.scope = format!("{}:{practice}", source.id);
            let task = if practice {
                &definition.example
            } else {
                &definition.initial
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let scope = if practice {
                &mut candidate.practice
            } else {
                &mut candidate.assignment
            };
            ui.push_id(("secure-workspace", &source.id, practice, challenge), |ui| {
            topology(ui, &scope.world);
            ui.add_enabled_ui(active,|ui|{
                if practice{lesson_presentation::practice_step(ui,&source,0);}
                settings(ui,scope,&mut self.metrics.port_security,&mut self.focus_input,&mut self.notice);
                if practice{lesson_presentation::practice_step(ui,&source,1);}
                let h=ui.text_style_height(&egui::TextStyle::Body);
                if ui.available_width()>=h*62.{
                    ui.columns(2,|columns|{replay(&mut columns[0],scope,&mut self.metrics.port_security);secure_table(&mut columns[1],scope);});
                }else{replay(ui,scope,&mut self.metrics.port_security);secure_table(ui,scope);}
                if practice{lesson_presentation::practice_step(ui,&source,2);}
                let r=ui.add(primary(ui,"Check work"));self.metrics.check=Some(r.rect);record(&mut self.metrics.port_security,"check",&r,ui);
                if r.clicked(){self.notice=scope.check(task).err();}
                controls::feedback(ui,r.clicked(),|ui|{
                if scope.checked&&scope.applied(){
                    let rows=task.checks(&scope.world);let passed=rows.iter().filter(|r|r.0).count();
                    let r=lesson_presentation::rich(ui,RichText::new(if passed==rows.len(){"All secure-port requirements match"}else{"Secure-port work needs attention"}).strong().color(if passed==rows.len(){palette(ui).success}else{palette(ui).error}));
                    self.metrics.port_security.result=Some((passed,rows.len(),r.rect));
                    for(ok,text)in rows{lesson_presentation::prose(ui,&format!("{} · {text}",if ok{"Matches"}else{"Needs work"}));}
                }else{lesson_presentation::prose(ui,if scope.applied(){"Not checked · compare the current configuration and traffic cases when ready."}else{"Pending settings · Apply settings before checking."});}
                });
                if practice{
                    let worked_help=controls::collapsing_header(ui,"Worked practice configuration · optional").show(ui,|ui|{
                        lesson_presentation::worked_rows(ui,&source);
                        lesson_presentation::prose(ui,"Apply worked example sets this practice's policy and recovery state. It leaves Check work for you and does not change the assignment.");
                        let r=ui.add(button("Apply worked example"));record(&mut self.metrics.port_security,"fill-example",&r,ui);if r.clicked(){self.notice=scope.fill(task).err();}
                    });
                    record(&mut self.metrics.port_security,"worked-example",&worked_help.header_response,ui);
                    let r=ui.add(button("Reset example"));record(&mut self.metrics.port_security,"reset-example",&r,ui);if r.clicked(){*scope=work::Scope::new(task);self.notice=None;}
                }
                let help = controls::collapsing_header(ui,"Help and simulation scope").show(ui,|ui|{
                    if !practice{
                        let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));if r.clicked(){a.hints+=1;self.dirty=true;}for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}
                        let worked = controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});
                        record(&mut self.metrics.port_security,"worked-assignment",&worked.header_response,ui);
                    }
                    lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}
                });
                record(&mut self.metrics.port_security,"help",&help.header_response,ui);
            });
            });
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::PortSecurity {
                            simulation: Box::new(candidate),
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
