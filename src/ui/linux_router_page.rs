//! Native virtual-router workspaces; bounded commands never reach the host.
use super::*;
use dlms_learning_lab::{linux_router_source as binding, linux_router_workspace as work};
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
fn topology(ui: &mut egui::Ui, s: &work::Scope) -> egui::Response {
    let interfaces = &s.state.host.configuration.interfaces;
    let lan = &interfaces[0];
    let wan = &interfaces[1];
    let address = |i: &dlms_learning_lab::linux_host::Interface| {
        i.address.as_ref().map_or("No address".into(), |a| {
            format!("{}/{}", a.address, a.prefix)
        })
    };
    let left = address(lan);
    let right = address(wan);
    let middle = if s.state.forwarding {
        "Forwarding on"
    } else {
        "Forwarding off"
    };
    controls::topology_chain(
        ui,
        &[
            [
                "LAN interface",
                &left,
                if lan.up { "lan0 · Up" } else { "lan0 · Down" },
            ],
            ["Linux router", "INPUT · FORWARD · OUTPUT", middle],
            [
                "WAN interface",
                &right,
                if wan.up { "wan0 · Up" } else { "wan0 · Down" },
            ],
        ],
        "Supplied virtual router · traffic addressed to the router uses INPUT; transit uses FORWARD.",
    )
}
fn editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    notice: &mut Option<String>,
    viewport_height: f32,
) {
    ui.strong("Saved configuration · simulated files");
    let mut file = s.file.clone();
    controls::form_row(ui, "File", |ui, width| {
        let r = controls::dropdown(ui, "router-file", width)
            .selected_text(file.path())
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for f in [work::File::Filter, work::File::Forwarding] {
                    let r = ui.selectable_value(&mut file, f.clone(), f.path());
                    m.record(ui, &format!("file:{}", f.path()), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "file", &r.response);
    });
    if file != s.file {
        apply(s, t, &work::Action::File { file }, notice);
    }
    let mut text = s.draft().to_owned();
    let line = ui.text_style_height(&egui::TextStyle::Monospace);
    let padding = 2. * f32::from(controls::PAD_Y);
    // Reserve part of the real parent viewport, then fit complete font rows.
    // Text/cursor metrics stay native; long configuration files scroll inside.
    let rows = ((viewport_height * 0.65 - padding - ui.spacing().scroll.allocated_width()) / line)
        .floor()
        .clamp(4., 10.) as usize;
    let height = line * rows as f32 + padding;
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let label = ui.label(s.file.path());
    let (outer, scroll) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("router-editor-scroll")
            .auto_shrink([false, false])
            .max_height(size.y)
            .min_scrolled_height(height)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt("router-editor")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(rows)
                        .char_limit(if s.file == work::File::Filter {
                            32768
                        } else {
                            4096
                        }),
                )
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let r = scroll.inner.labelled_by(label.id);
    let visible = r.rect.intersect(scroll.inner_rect);
    m.fields.push(("editor".into(), visible, r.id));
    m.controls.push(("editor".into(), visible, ui.clip_rect()));
    if r.changed() {
        apply(s, t, &work::Action::Edit { text }, notice);
    }
    ui.horizontal_wrapped(|ui| {
        for (key, label, a) in [
            ("save", "Save file", work::Action::Save),
            ("reload-editor", "Reload editor", work::Action::ReloadDraft),
        ] {
            let r = ui.add(button(label));
            m.record(ui, key, &r);
            if r.clicked() {
                apply(s, t, &a, notice);
            }
        }
    });
    let saved = if s.file == work::File::Filter {
        &s.state.filter_file
    } else {
        &s.state.forward_file
    };
    lesson_presentation::prose(
        ui,
        if s.draft() == saved {
            "Draft saved · use an explicit command to apply it."
        } else {
            "Unsaved draft · Save file writes this virtual file. Reload editor discards this draft."
        },
    );
}
fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    m: &mut Metrics,
    notice: &mut Option<String>,
    phase: (bool, &mut bool),
) {
    let (practice, focus) = phase;
    ui.strong("Router terminal · simulated commands");
    if practice {
        lesson_presentation::practice_step(ui, source, if t.files { 1 } else { 0 });
        let commands = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            let key = egui::Id::new(("router-command-choice", &source.id));
            let mut n = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, width| {
                let r = controls::dropdown(ui, "router-command-choice", width)
                    .selected_text(&commands[n].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (at, c) in commands.iter().enumerate() {
                            ui.selectable_value(&mut n, at, &c.command);
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "example-command", &r.response);
            });
            lesson_presentation::prose(ui, &commands[n].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                apply(
                    s,
                    t,
                    &work::Action::Input {
                        text: commands[n].command.clone(),
                    },
                    notice,
                );
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, n));
        }
    }
    let height = ui.text_style_height(&egui::TextStyle::Monospace)
        * s.output.lines().count().clamp(4, 10) as f32;
    let output=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
  ui.strong("Actual command results");
  controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("router-command-output").auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{if s.output.is_empty(){lesson_presentation::prose(ui,"No command has run. Inspect the running rules, forwarding setting or routes to begin.");}else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}});
 });
    m.record(ui, "output-viewport", &output.response);
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("router-command-input", &source.id, practice)),
            2048,
            ui.is_enabled(),
            "Command · Enter to run",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(s, t, &work::Action::Input { text }, notice);
        }
        if *focus && ui.is_enabled() {
            r.response.request_focus();
            *focus = false;
        }
        run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Keep one command within 2 KiB; previous input retained.",
            );
        }
    };
    let width = controls::button_width(ui, "Run command");
    let clicked = if ui.available_width()
        > width + ui.text_style_height(&egui::TextStyle::Body) * 10. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let width = (ui.available_width() - width - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(width, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| field(ui, m),
            );
            let r = ui.add(button("Run command"));
            m.record(ui, "run", &r);
            r.clicked()
        })
        .inner
    } else {
        field(ui, m);
        let r = ui.add(button("Run command"));
        m.record(ui, "run", &r);
        r.clicked()
    };
    if run || clicked {
        let line = s.input.clone();
        apply(s, t, &work::Action::Command { line }, notice);
        *focus = true;
    }
}
fn traffic(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    ui.add_space(8.);
    ui.strong("Traffic cases · read-only policy analysis");
    let mut selected = s.selected_case;
    controls::form_row(ui, "Traffic case", |ui, width| {
        let r = controls::dropdown(ui, "router-traffic-case", width)
            .selected_text(&t.cases[selected].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (at, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, at, &c.label);
                    m.record(ui, &format!("case:{at}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "case", &r.response);
    });
    if selected != s.selected_case {
        apply(s, t, &work::Action::SelectCase { index: selected }, notice);
    }
    let c = &t.cases[s.selected_case];
    lesson_presentation::monospace(
        ui,
        &format!(
            "{} -> {} · {} · destination port {}",
            c.header.source,
            c.header.destination,
            match c.header.protocol {
                dlms_learning_lab::linux_filter::Protocol::Tcp => "TCP",
                dlms_learning_lab::linux_filter::Protocol::Udp => "UDP",
                dlms_learning_lab::linux_filter::Protocol::Icmp => "ICMP",
            },
            c.header
                .destination_port
                .map_or("none".into(), |n| n.to_string())
        ),
        true,
    );
    lesson_presentation::prose(
        ui,
        if c.allow {
            "Required outcome: allowed"
        } else {
            "Required outcome: blocked"
        },
    );
    let r = ui.add(button("Trace policy"));
    m.record(ui, "trace", &r);
    if r.clicked() {
        apply(s, t, &work::Action::Trace, notice);
    }
}
impl LabApp {
    pub(super) fn router_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (engine::Goal::LinuxRouter { definition }, engine::State::LinuxRouter { simulation }) =
            (&source.goal, &original.state)
        else {
            return;
        };
        let first = egui::Id::new(("router-first", &source.id, original.visited));
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
        if let Err(e) = definition.bound(simulation) {
            lesson_presentation::prose(ui, &e);
            return;
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
                    self.metrics.router.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            let intro =
                lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .router
                .record(ui, "introduction", &intro.objective);
            lesson_presentation::worked(
                ui,
                "Separate worked router · use Try example to investigate and change it.",
            );
            let worked = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics.router.record(ui, "worked", &worked.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.router.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.assignment, &mut next.assignment)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics.router.record(ui, "assignment", &r);
            }
            let viewport_height = ui.clip_rect().intersect(ui.ctx().content_rect()).height();
            ui.add_enabled_ui(active, |ui| {
                let r = topology(ui, s);
                self.metrics.router.record(ui, "topology", &r);
                if t.files {
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 0);
                    }
                    if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 54. {
                        ui.columns(2, |cols| {
                            editor(&mut cols[0], t, s, &mut self.metrics.router, &mut self.notice, viewport_height);
                            console(&mut cols[1], t, s, &source, &mut self.metrics.router, &mut self.notice, (practice, &mut self.focus_input));
                        });
                    } else {
                        editor(ui, t, s, &mut self.metrics.router, &mut self.notice, viewport_height);
                        ui.add_space(12.);
                        console(ui, t, s, &source, &mut self.metrics.router, &mut self.notice, (practice, &mut self.focus_input));
                    }
                } else {
                    console(ui, t, s, &source, &mut self.metrics.router, &mut self.notice, (practice, &mut self.focus_input));
                }
                traffic(ui, t, s, &mut self.metrics.router, &mut self.notice);
                if practice {
                    if !t.files {
                        lesson_presentation::practice_step(ui, &source, 1);
                    }
                    lesson_presentation::practice_step(ui, &source, 2);
                }
                let r = ui.add(primary(ui, if practice { "Check example" } else { "Check work" }));
                self.metrics.router.record(ui, "check", &r);
                if r.clicked() {
                    apply(s, t, &work::Action::Check, &mut self.notice);
                }
                if s.checked {
                    let checks = t.checks(s);
                    let passed = checks.iter().filter(|c| c.passed).count();
                    let r = ui.strong(if passed == checks.len() {
                        if practice { "Example outcomes passed" } else { "Assignment complete" }.into()
                    } else {
                        format!("Not complete · {passed}/{} outcomes pass", checks.len())
                    });
                    self.metrics.router.result = Some((passed, checks.len(), r.rect));
                    self.metrics.router.record(ui, "result", &r);
                    for c in checks.iter().filter(|c| !c.passed) {
                        let r = lesson_presentation::prose(ui, &format!("Needs work · {}\n{}", c.label, c.detail));
                        self.metrics.router.record(ui, "failed-case", &r);
                    }
                    controls::collapsing_header(ui, "Inspect passing checks").show(ui, |ui| {
                        for c in checks.iter().filter(|c| c.passed) {
                            lesson_presentation::prose(ui, &format!("Passed · {}\n{}", c.label, c.detail));
                        }
                    });
                    if passed == checks.len() {
                        lesson_presentation::prose(ui, if practice {
                            "The example now meets both the policy comparison and the supplied traffic outcomes. Your assignment is separate."
                        } else { &source.explanation });
                    }
                }
            });
            controls::collapsing_header(ui, "Optional help and supported commands").show(
                ui,
                |ui| {
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
                        controls::collapsing_header(
                            ui,
                            "Worked approach · reveals assignment guidance",
                        )
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        });
                    } else {
                        controls::collapsing_header(ui, "Show worked answer · example only").show(
                            ui,
                            |ui| {
                                lesson_presentation::wrapped_worked_commands(ui, &source);
                            },
                        );
                    }
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                },
            );
        }
        if active && next != before {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::LinuxRouter {
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
