//! AP/controller forwarding practice. Native fields; no IOS or host execution.
use super::wlan_page::row;
use super::*;
use dlms_learning_lab::{
    engine::State,
    wlan_paths::{ApMode, Forwarding},
    wlan_paths_workspace as work,
};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub choices: Vec<(String, egui::Rect, egui::Rect)>,
    pub summary: Option<String>,
    pub worked_answer_visible: bool,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn mode(v: ApMode) -> &'static str {
    match v {
        ApMode::Local => "Local",
        ApMode::FlexConnect => "FlexConnect",
    }
}
fn forwarding(v: Forwarding) -> &'static str {
    match v {
        Forwarding::Central => "Central · at controller",
        Forwarding::Local => "Local · at AP",
    }
}
fn field(
    ui: &mut egui::Ui,
    key: &str,
    label: &str,
    value: &mut String,
    limit: usize,
    hint: &str,
    m: &mut Metrics,
) -> bool {
    let r = row(ui, label, |ui, width, id| {
        ui.add(
            controls::singleline_fitting(ui, value, width.min(284.))
                .font(egui::TextStyle::Monospace)
                .char_limit(limit)
                .hint_text(hint),
        )
        .labelled_by(id)
    });
    m.record(ui, key, &r);
    m.fields.push((key.into(), r.rect, r.id, ui.clip_rect()));
    r.changed()
}
fn topology(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let native = format!("Native VLAN {} · {}", d.fields.native, mode(d.fields.mode));
    let controller = format!(
        "Native VLAN {} · tagged client VLANs",
        t.initial.controller_native
    );
    let cards = [
        [
            "Access point",
            t.initial.fabric.hosts[&t.initial.ap].label.as_str(),
            native.as_str(),
        ],
        [
            "Wired access switch",
            "Supplied links and VLANs",
            "Editable attachments below",
        ],
        [
            "Controller",
            t.initial.fabric.hosts[&t.initial.controller].label.as_str(),
            controller.as_str(),
        ],
    ];
    let r = controls::topology_chain(
        ui,
        &cards,
        "AP and controller use the supplied switch. Management and client data can share one cable.",
    );
    m.record(ui, "topology", &r);
}
fn ap_settings(ui: &mut egui::Ui, t: &work::Task, d: &mut work::Draft, m: &mut Metrics) {
    ui.strong("AP and WLAN settings");
    let mut changed = false;
    if t.edit_mode {
        let old = d.fields.mode;
        row(ui, "AP mode", |ui, width, label| {
            let r = controls::dropdown(ui, "ap-mode", width.min(284.))
                .selected_text(mode(d.fields.mode))
                .show_ui(ui, |ui| {
                    for v in [ApMode::Local, ApMode::FlexConnect] {
                        let r = ui.selectable_value(&mut d.fields.mode, v, mode(v));
                        m.choices
                            .push((format!("mode-{v:?}"), r.rect, ui.clip_rect()));
                    }
                });
            r.response.clone().labelled_by(label);
            m.record(ui, "mode", &r.response);
        });
        changed |= old != d.fields.mode;
    } else {
        row(ui, "AP mode", |ui, _, _| ui.label(mode(d.fields.mode)));
    }
    if t.edit_native {
        changed |= field(
            ui,
            "native",
            "AP native VLAN",
            &mut d.fields.native,
            4,
            "1–4094",
            m,
        );
    } else {
        row(ui, "AP native VLAN", |ui, _, _| {
            ui.monospace(&d.fields.native)
        });
    }
    for (value, initial) in d.fields.wlans.iter_mut().zip(&t.initial.wlans) {
        let edit = t.edit_wlans.iter().find(|e| e.id == value.id);
        ui.add_space(8.);
        if edit.is_none() {
            ui.label(format!(
                "{} · protected: {} · controller VLAN {}",
                initial.label,
                forwarding(value.forwarding),
                value.controller_vlan
            ));
            continue;
        }
        ui.strong(&initial.label);
        if edit.is_some_and(|e| e.forwarding) {
            let old = value.forwarding;
            row(ui, "WLAN switching", |ui, width, label| {
                let r = controls::dropdown(ui, (&value.id, "forwarding"), width.min(284.))
                    .selected_text(forwarding(value.forwarding))
                    .show_ui(ui, |ui| {
                        for v in [Forwarding::Central, Forwarding::Local] {
                            let r = ui.selectable_value(&mut value.forwarding, v, forwarding(v));
                            m.choices.push((
                                format!("forwarding-{}-{v:?}", value.id),
                                r.rect,
                                ui.clip_rect(),
                            ));
                        }
                    });
                r.response.clone().labelled_by(label);
                m.record(ui, &format!("forwarding-{}", value.id), &r.response);
            });
            changed |= old != value.forwarding;
        } else {
            row(ui, "WLAN switching", |ui, _, _| {
                ui.label(forwarding(value.forwarding))
            });
        }
        if edit.is_some_and(|e| e.local_vlan) {
            changed |= field(
                ui,
                &format!("local-{}", value.id),
                "Local VLAN",
                &mut value.local_vlan,
                4,
                "1–4094",
                m,
            );
        }
        if edit.is_some_and(|e| e.controller_vlan) {
            changed |= field(
                ui,
                &format!("controller-{}", value.id),
                "Controller VLAN",
                &mut value.controller_vlan,
                4,
                "1–4094",
                m,
            );
        }
        if edit.is_some_and(|e| !e.local_vlan && !e.controller_vlan) {
            row(ui, "Supplied mappings", |ui, _, _| {
                ui.label(format!(
                    "local {} · controller {}",
                    value.local_vlan, value.controller_vlan
                ))
            });
        }
    }
    if changed {
        d.edited();
    }
}
fn port_table(ui: &mut egui::Ui, t: &work::Task, d: &mut work::Draft, m: &mut Metrics) {
    ui.strong("Permitted switch attachments");
    let line = ui.text_style_height(&egui::TextStyle::Body);
    let cols = [
        (line * 11.).max(180.),
        controls::button_width(ui, "Up").max(60.),
        controls::button_width(ui, "Access").max(112.),
        (line * 6.).max(110.),
    ];
    let minimum = cols.iter().sum::<f32>() + 180. + 4. * controls::GAP;
    let wide = ui.available_width() >= minimum;
    let allowed = (ui.available_width() - cols.iter().sum::<f32>() - 4. * controls::GAP).max(180.);
    let mut changed = false;
    egui::Grid::new("ap-switch-attachments")
        .num_columns(if wide { 5 } else { 2 })
        .spacing([controls::GAP, 8.])
        .min_row_height(controls::height(ui))
        .show(ui, |ui| {
            if wide {
                for (text, width) in [
                    "Switch / port",
                    "State",
                    "Mode",
                    "VLAN / native",
                    "Allowed VLANs",
                ]
                .into_iter()
                .zip(cols.into_iter().chain([allowed]))
                {
                    controls::label(ui, RichText::new(text).strong(), width);
                }
                ui.end_row();
            }
            for edit in &t.edit_ports {
                let p = d
                    .fields
                    .ports
                    .switches
                    .get_mut(&edit.switch)
                    .unwrap()
                    .ports
                    .get_mut(&edit.port)
                    .unwrap();
                let name = format!("{}/{}", edit.switch, edit.port);
                let key = format!("{}-{}", edit.switch, edit.port);
                if wide {
                    controls::label(ui, RichText::new(&name).monospace(), cols[0]);
                } else {
                    ui.strong(&name);
                    ui.end_row();
                }
                if !wide {
                    controls::label(ui, "State", controls::form_label_width(ui));
                }
                controls::cell(ui, cols[1], |ui| {
                    if edit.up {
                        let r = ui.checkbox(&mut p.up, "Up");
                        changed |= r.changed();
                        m.record(ui, &format!("up-{key}"), &r);
                    } else {
                        ui.label(if p.up { "Up" } else { "Down" });
                    }
                });
                if !wide {
                    ui.end_row();
                    controls::label(ui, "Mode", controls::form_label_width(ui));
                }
                controls::cell(
                    ui,
                    if wide { cols[2] } else { ui.available_width() },
                    |ui| {
                        if edit.mode {
                            let old = p.trunk;
                            let r = controls::dropdown(
                                ui,
                                (&key, "mode"),
                                if wide {
                                    cols[2]
                                } else {
                                    ui.available_width().min(284.)
                                },
                            )
                            .selected_text(if p.trunk { "Trunk" } else { "Access" })
                            .show_ui(ui, |ui| {
                                for (v, label) in [(false, "Access"), (true, "Trunk")] {
                                    let r = ui.selectable_value(&mut p.trunk, v, label);
                                    m.choices.push((
                                        format!("port-mode-{key}-{label}"),
                                        r.rect,
                                        ui.clip_rect(),
                                    ));
                                }
                            });
                            changed |= p.trunk != old;
                            m.record(ui, &format!("port-mode-{key}"), &r.response);
                        } else {
                            ui.label(if p.trunk { "Trunk" } else { "Access" });
                        }
                    },
                );
                if !wide {
                    ui.end_row();
                    controls::label(
                        ui,
                        if p.trunk {
                            "Native VLAN"
                        } else {
                            "Access VLAN"
                        },
                        controls::form_label_width(ui),
                    );
                }
                controls::cell(
                    ui,
                    if wide { cols[3] } else { ui.available_width() },
                    |ui| {
                        let value = if p.trunk {
                            &mut p.native
                        } else {
                            &mut p.access
                        };
                        if edit.mode {
                            let r = ui.add(
                                controls::singleline_fitting(
                                    ui,
                                    value,
                                    if wide {
                                        cols[3]
                                    } else {
                                        ui.available_width().min(284.)
                                    },
                                )
                                .font(egui::TextStyle::Monospace)
                                .char_limit(4)
                                .hint_text("1–4094"),
                            );
                            changed |= r.changed();
                            m.record(ui, &format!("port-vlan-{key}"), &r);
                            m.fields.push((
                                format!("port-vlan-{key}"),
                                r.rect,
                                r.id,
                                ui.clip_rect(),
                            ));
                        } else {
                            ui.monospace(value);
                        }
                    },
                );
                if !wide {
                    ui.end_row();
                    controls::label(ui, "Allowed VLANs", controls::form_label_width(ui));
                }
                controls::cell(
                    ui,
                    if wide { allowed } else { ui.available_width() },
                    |ui| {
                        if p.trunk && edit.mode {
                            let r = ui.add(
                                controls::singleline_fitting(
                                    ui,
                                    &mut p.allowed,
                                    if wide {
                                        allowed
                                    } else {
                                        ui.available_width().min(384.)
                                    },
                                )
                                .font(egui::TextStyle::Monospace)
                                .char_limit(384)
                                .hint_text("VLANs, commas"),
                            );
                            changed |= r.changed();
                            m.record(ui, &format!("port-allowed-{key}"), &r);
                            m.fields.push((
                                format!("port-allowed-{key}"),
                                r.rect,
                                r.id,
                                ui.clip_rect(),
                            ));
                        } else {
                            ui.label(if p.trunk {
                                p.allowed.as_str()
                            } else {
                                "One access VLAN"
                            });
                        }
                    },
                );
                ui.end_row();
            }
        });
    if changed {
        d.edited();
    }
}
fn cases(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let r = ui.strong("Required client-data paths");
    m.record(ui, "cases", &r);
    let actual = d.checked().then(|| t.checks(&d.fields));
    let blocked = actual.as_ref().is_some_and(|rows| {
        rows.iter().any(|r| {
            !r.correct && ["management", "configuration", "definition"].contains(&r.id.as_str())
        })
    });
    let management = ui.vertical(|ui| {
        ui.strong("Management transport");
        ui.label(format!(
            "Required: supplied native VLAN {} in both directions",
            t.initial.controller_native
        ));
        if let Some(r) = actual
            .as_ref()
            .and_then(|rows| rows.iter().find(|r| r.id == "management"))
        {
            ui.label(
                RichText::new(if r.correct {
                    "Meets requirement"
                } else {
                    "Needs attention"
                })
                .color(if r.correct {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            if !r.correct {
                lesson_presentation::prose(ui, &r.feedback);
            }
        }
        ui.separator();
    });
    m.record(ui, "case-management", &management.response);
    for c in &t.cases {
        let card=ui.vertical(|ui| {
        ui.strong(&c.label);
        match c.expected {
            work::Expected::Path {
                forwarding: kind,
                vlan,
            } => {
                ui.label(format!(
                    "Required: {} · VLAN {vlan} · both directions",
                    forwarding(kind)
                ));
            }
            work::Expected::Isolated => {
                ui.label("Required: no staff data to the visitor service, with the physical path kept working");
            }
        }
        if let Some(row) = actual
            .as_ref()
            .and_then(|rows| rows.iter().find(|r| r.id == c.id))
        {
            let (text, color) = if !row.evaluated {
                ("Not evaluated", palette(ui).muted)
            } else if row.correct {
                ("Meets requirement", palette(ui).success)
            } else {
                ("Needs attention", palette(ui).error)
            };
            ui.label(RichText::new(text).color(color));
            if !row.correct && (row.evaluated || !blocked) {
                lesson_presentation::prose(ui, &row.feedback);
            }
        }
        ui.separator();
        });
        m.record(ui, &format!("case-{}", c.id), &card.response);
    }
}
fn feedback(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    if !d.checked() {
        let r = ui.label("Not checked since editing");
        m.record(ui, "result", &r);
        m.summary = Some("unchecked".into());
        return;
    }
    let rows = t.checks(&d.fields);
    let failed = rows.iter().filter(|r| !r.correct || !r.evaluated).count();
    if failed == 0 {
        let r = ui.label(RichText::new("All required paths verified").color(palette(ui).success));
        m.record(ui, "result", &r);
        m.summary = Some("pass".into());
    } else {
        let r = ui.label(
            RichText::new(format!("Needs attention · {failed} requirements remain"))
                .color(palette(ui).error),
        );
        m.record(ui, "result", &r);
        m.summary = Some("fail".into());
    }
    // Client-specific results sit beside their requirements. Show only configuration/control here.
    for r in rows
        .iter()
        .filter(|r| !r.correct && (r.id == "configuration" || r.id == "definition"))
    {
        let detail = ui.vertical(|ui| lesson_presentation::prose(ui, &r.feedback));
        m.record(ui, "configuration-feedback", &detail.response);
    }
}
fn worked(ui: &mut egui::Ui, t: &work::Task) {
    ui.label(format!(
        "AP mode: {} · native VLAN {}",
        mode(t.reference.mode),
        t.reference.native
    ));
    for e in &t.edit_wlans {
        let f = t.reference.wlans.iter().find(|f| f.id == e.id).unwrap();
        ui.label(format!(
            "{}: {} · local VLAN {} · controller VLAN {}",
            t.initial.wlans.iter().find(|w| w.id == e.id).unwrap().label,
            forwarding(f.forwarding),
            f.local_vlan,
            f.controller_vlan
        ));
    }
    for e in &t.edit_ports {
        let p = &t.reference.ports.switches[&e.switch].ports[&e.port];
        ui.label(format!(
            "{}/{}: {} · VLAN/native {} · allowed {}",
            e.switch,
            e.port,
            if p.trunk { "Trunk" } else { "Access" },
            if p.trunk { &p.native } else { &p.access },
            p.allowed
        ));
    }
}
impl LabApp {
    pub(super) fn wlan_paths_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let State::WlanPaths { simulation } = &attempt.state else {
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
        self.metrics.wlan_paths = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    self.metrics.wlan_paths.record(ui, key, &r);
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
        ui.push_id((i, view as u8, "wlan-paths"), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Compare AP mode, WLAN forwarding and the supplied VLAN attachments.",
                    Vec::<(String, String)>::new(),
                );
                lesson_presentation::worked(ui, "A different site demonstrates the method.");
                lesson_presentation::worked_rows(ui, &source);
                ui.horizontal_wrapped(|ui| {
                    for (v, key, label) in [
                        (teaching::View::Example, "try", "Try this example"),
                        (teaching::View::Assignment, "start", "Open full assignment"),
                    ] {
                        let r = ui.add_enabled(active, button(label));
                        self.metrics.wlan_paths.record(ui, key, &r);
                        if r.clicked() {
                            next.view = v;
                            self.lesson_scroll_reset = true;
                        }
                    }
                });
            } else {
                let practice = view == teaching::View::Example;
                let (t, d) = if practice {
                    (&def.example, &mut next.practice)
                } else {
                    (&def.initial, &mut next.assignment)
                };
                if practice {
                    lesson_presentation::practice_step(ui, &source, 0);
                } else {
                    ui.label(RichText::new("YOUR TASK").small().color(palette(ui).accent));
                    let goal=lesson_presentation::entry(&source).map(|e|e.goal.as_str()).unwrap_or(&source.summary);
                    ui.label(RichText::new(goal).size(22.));
                    lesson_presentation::prose(ui,"Meet every required path beside the controls. Edit only the displayed settings; preserve the supplied endpoints, cables and other ports. Wi-Fi names, security and IP settings are supplied and working.");
                }
                topology(ui, t, d, &mut self.metrics.wlan_paths);
                ui.add_enabled_ui(active, |ui| {
                    let column_min = controls::form_label_width(ui)
                        + ui.text_style_height(&egui::TextStyle::Body) * 9.
                        + 2. * controls::PAD_X as f32
                        + controls::GAP;
                    if ui.available_width() >= column_min * 2. + controls::GAP {
                        ui.columns(2, |cols| {
                            ap_settings(&mut cols[0], t, d, &mut self.metrics.wlan_paths);
                            cases(&mut cols[1], t, d, &mut self.metrics.wlan_paths);
                        });
                    } else {
                        ap_settings(ui, t, d, &mut self.metrics.wlan_paths);
                        ui.separator();
                        cases(ui, t, d, &mut self.metrics.wlan_paths);
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 1);
                    }
                    port_table(ui, t, d, &mut self.metrics.wlan_paths);
                    if practice {
                        let r = ui.add(button("Fill worked configuration"));
                        self.metrics.wlan_paths.record(ui, "fill", &r);
                        if r.clicked() {
                            self.notice = d.fill(t).err();
                        }
                    }
                    let r = ui.add(primary(ui, "Check paths"));
                    self.metrics.wlan_paths.record(ui, "check", &r);
                    if r.clicked() {
                        self.notice = d.check(t).err();
                    }
                    feedback(ui, t, d, &mut self.metrics.wlan_paths);
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                        let r = ui.add(button("Reset example only"));
                        self.metrics.wlan_paths.record(ui, "reset-example", &r);
                        if r.clicked() {
                            d.reset(t);
                            self.notice = None;
                        }
                    }
                    controls::collapsing_header(ui, "Hints and explanation · optional").show(
                        ui,
                        |ui| {
                            let attempt = self.progress.attempts.get_mut(&source.id).unwrap();
                            let r = ui.add_enabled(
                                active && attempt.hints < source.hints.len(),
                                button("Show a hint"),
                            );
                            if r.clicked() {
                                attempt.hints += 1;
                                self.dirty = true;
                            }
                            for hint in source.hints.iter().take(attempt.hints) {
                                lesson_presentation::prose(ui, hint);
                            }
                            controls::collapsing_header(ui, "Why does this work?")
                                .show(ui, |ui| lesson_presentation::prose(ui, &source.explanation));
                            controls::collapsing_header(
                                ui,
                                "Show worked answer · reveals one configuration",
                            )
                            .show(ui, |ui| {
                                self.metrics.wlan_paths.worked_answer_visible = true;
                                worked(ui, t);
                            });
                            controls::collapsing_header(ui, "Simulation scope and references")
                                .show(ui, |ui| {
                                    lesson_presentation::prose(ui, work::HELP);
                                    lesson_presentation::prose(ui, &source.simplifications);
                                    for url in &source.sources {
                                        ui.hyperlink_to(url, url);
                                    }
                                });
                        },
                    );
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = State::WlanPaths {
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
