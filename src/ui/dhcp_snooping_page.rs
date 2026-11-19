//! Native policy, supplied traffic and actual lease observations.
use super::*;
use dlms_learning_lab::{dhcp_snooping as core, dhcp_snooping_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub scope: String,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub replay_output: Option<egui::Rect>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn topology(ui: &mut egui::Ui, w: &core::World) {
    topology_with_caption(
        ui,
        w,
        "Server trust applies to incoming messages on a port; it does not authenticate a device.",
    );
}
pub(super) fn topology_with_caption(ui: &mut egui::Ui, w: &core::World, caption: &str) {
    use dlms_learning_lab::switching::{Jack, Mode};
    let state = w.forwarding();
    let f = state.fabric();
    let (switch, s) = f.switches.iter().next().unwrap();
    let mut cards = vec![];
    for (host, endpoint) in &f.hosts {
        let port = f
            .cables
            .iter()
            .find_map(|c| match (&c.a, &c.b) {
                (Jack::Host { host: h }, Jack::Switch { port, .. })
                | (Jack::Switch { port, .. }, Jack::Host { host: h })
                    if h == host =>
                {
                    Some(port)
                }
                _ => None,
            })
            .unwrap();
        let Mode::Access { vlan } = s.ports[port].mode else {
            unreachable!()
        };
        cards.push([
            endpoint.label.clone(),
            state.address(host).unwrap().text(),
            format!("{port} · VLAN {vlan}"),
        ]);
    }
    let refs = cards
        .iter()
        .map(|c| [c[0].as_str(), c[1].as_str(), c[2].as_str()])
        .collect::<Vec<_>>();
    if !controls::topology_star(
        ui,
        [&s.label, switch, "Supplied access network"],
        &refs,
        caption,
    ) {
        for c in cards {
            lesson_presentation::prose(ui, &c.join(" · "));
        }
    }
}
fn policy(
    ui: &mut egui::Ui,
    scope: &mut work::Scope,
    m: &mut Metrics,
    focus: &mut bool,
    notice: &mut Option<String>,
) {
    let before = scope.draft.clone();
    let h = ui.text_style_height(&egui::TextStyle::Body);
    lesson_presentation::rich(ui, RichText::new("DHCP inspection policy").strong());
    ui.horizontal_wrapped(|ui| {
        let r = ui.checkbox(&mut scope.draft.enabled, "Enable DHCP snooping");
        record(m, "enabled", &r, ui);
        let r = ui.checkbox(
            &mut scope.draft.verify_source,
            "Compare the two client MAC fields",
        );
        record(m, "verify-source", &r, ui);
    });
    controls::form_row(ui, "Inspected VLANs", |ui, available| {
        let width = available.min(h * 24.);
        let r = ui.add(
            controls::singleline_fitting(ui, &mut scope.draft.vlans, width)
                .id(egui::Id::new(("dhcp-policy-field", &m.scope, "vlans")))
                .hint_text("Comma-separated VLAN IDs"),
        );
        if *focus {
            r.request_focus();
            *focus = false;
        }
        m.fields
            .push(("vlans".into(), r.rect, r.id, ui.clip_rect()));
    });
    let wide = ui.available_width() >= h * 39.;
    if wide {
        egui::Grid::new("dhcp-trust-table")
            .num_columns(2)
            .min_col_width(h * 12.)
            .show(ui, |ui| {
                lesson_presentation::rich(ui, RichText::new("Port").strong());
                lesson_presentation::rich(ui, RichText::new("Incoming DHCP messages").strong());
                ui.end_row();
                for p in &mut scope.draft.ports {
                    controls::table_text(ui, &p.port, h * 12.);
                    let label = "Trust incoming server replies";
                    let r = ui.checkbox(&mut p.trusted, label);
                    record(m, &format!("trust-{}", p.port), &r, ui);
                    ui.end_row();
                }
            });
    } else {
        for p in &mut scope.draft.ports {
            controls::form_row(ui, &p.port, |ui, _| {
                // Horizontal form values default to Extend in egui. Let this
                // descriptive checkbox wrap within its reserved value column.
                ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
                let label = "Trust incoming server replies";
                let r = ui.checkbox(&mut p.trusted, label);
                record(m, &format!("trust-{}", p.port), &r, ui);
            });
        }
    }
    if scope.draft != before {
        scope.checked = false;
        scope.output.clear();
    }
    let r = ui.add(button("Apply policy"));
    record(m, "apply", &r, ui);
    if r.clicked() {
        *notice = scope.apply().err();
    }
    if let Some(e) = notice.as_deref() {
        lesson_presentation::rich(ui, RichText::new(e).color(palette(ui).error));
    }
    lesson_presentation::prose(
        ui,
        if scope.applied() {
            "Current policy applied."
        } else {
            "Changes are not applied yet."
        },
    );
}
fn replay(
    ui: &mut egui::Ui,
    scope: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    lesson_presentation::rich(
        ui,
        RichText::new("Investigate with supplied traffic").strong(),
    );
    let before = scope.sample;
    let visible_width = ui
        .available_width()
        .min((ui.clip_rect().right() - ui.next_widget_position().x).max(0.));
    let r = controls::dropdown(ui, "dhcp-sample", visible_width)
        .selected_text(&t.cases[scope.sample].label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (i, c) in t.cases.iter().enumerate() {
                let r = ui.selectable_value(&mut scope.sample, i, &c.label);
                record(m, &format!("sample-{i}"), &r, ui);
            }
        });
    record(m, "sample", &r.response, ui);
    if before != scope.sample {
        scope.checked = false;
        scope.output.clear();
    }
    let case = &t.cases[scope.sample];
    // The diagnostic sample's two essential fields belong beside Replay,
    // rather than inside optional protocol detail.
    if let [work::Step::Message { packet, .. }] = case.steps.as_slice() {
        controls::form_row(ui, "Ethernet source MAC", |ui, _| {
            lesson_presentation::rich(ui, RichText::new(packet.source.text()).monospace());
        });
        controls::form_row(ui, "DHCP client MAC", |ui, _| {
            lesson_presentation::rich(ui, RichText::new(packet.client.text()).monospace());
        });
    }
    let detail = controls::collapsing_header(ui, "Messages in this sample").show(ui, |ui| {
        for (i, s) in case.steps.iter().enumerate() {
            match s {
                work::Step::Message { host, packet, .. } => {
                    lesson_presentation::prose(
                        ui,
                        &format!(
                            "{}. {} · {}",
                            i + 1,
                            host.0,
                            work::message_label(packet.message)
                        ),
                    );
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!(
                            "Ethernet source: {}   DHCP client MAC: {}",
                            packet.source.text(),
                            packet.client.text()
                        ))
                        .monospace(),
                    );
                }
                work::Step::Advance { seconds } => {
                    lesson_presentation::prose(
                        ui,
                        &format!("{}. Advance {seconds} simulated seconds.", i + 1),
                    );
                }
            }
        }
    });
    record(m, "messages", &detail.header_response, ui);
    lesson_presentation::prose(
        ui,
        "Replay starts a fresh sample network and replaces earlier replay observations. Your policy draft stays in place.",
    );
    let r = ui.add_enabled(scope.applied(), button("Replay sample"));
    record(m, "replay", &r, ui);
    if r.clicked() {
        *notice = scope.run_sample(t).err();
    }
    lesson_presentation::rich(ui, RichText::new("Actual replay results").strong());
    let h = ui.text_style_height(&egui::TextStyle::Body);
    let height = h * 8.;
    let (outer, _) = controls::fixed_viewport(ui, egui::vec2(visible_width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt("dhcp-replay-output")
                .max_height(height)
                .min_scrolled_height(height)
                .auto_shrink([false, false])
                .stick_to_bottom(true),
            |ui| {
                if scope.output.is_empty() {
                    lesson_presentation::prose(ui, "Not run for these settings and sample yet.");
                }
                for line in &scope.output {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(&line.text).color(if line.error {
                            palette(ui).error
                        } else {
                            palette(ui).text
                        }),
                    );
                }
            },
        );
    });
    m.replay_output = Some(outer);
}
fn results(ui: &mut egui::Ui, scope: &work::Scope, task: &work::Task, m: &mut Metrics) {
    if !scope.checked || !scope.applied() {
        lesson_presentation::prose(
            ui,
            if scope.applied() {
                "Not checked · compare your policy with the traffic requirements when ready."
            } else {
                "Apply current policy before checking."
            },
        );
        return;
    }
    let reports = task.reports(&scope.world);
    let passed = reports.iter().filter(|r| r.passed()).count();
    let all = passed == reports.len();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if all {
            "All DHCP traffic checks match"
        } else {
            "DHCP policy needs attention"
        })
        .strong()
        .color(if all {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, reports.len(), r.rect));
    for (i, report) in reports.iter().enumerate() {
        let status = if report.passed() {
            "Matches"
        } else {
            "Needs work"
        };
        let label = report.label.as_deref().unwrap_or("Supplied workspace");
        lesson_presentation::rich(ui, RichText::new(format!("{status} · {label}")).strong());
        if let Some(first) = report.failures.first() {
            lesson_presentation::prose(ui, first);
            if report.failures.len() > 1 {
                ui.push_id(("dhcp-case-detail", i), |ui| {
                    controls::collapsing_header(ui, "More observations in this case").show(
                        ui,
                        |ui| {
                            for text in report.failures.iter().skip(1) {
                                lesson_presentation::prose(ui, text);
                            }
                        },
                    );
                });
            }
        }
    }
}
fn leases(ui: &mut egui::Ui, scope: &work::Scope) {
    lesson_presentation::rich(
        ui,
        RichText::new(if scope.output.is_empty() {
            "Earlier lease observations"
        } else {
            "Observed address records (bindings)"
        })
        .strong(),
    );
    if scope.world.bindings().is_empty() {
        lesson_presentation::prose(ui, "No address has been recorded in this sample state.");
        return;
    }
    let h = ui.text_style_height(&egui::TextStyle::Body);
    for b in scope.world.bindings() {
        lesson_presentation::rich(
            ui,
            RichText::new(format!("{}  →  {}", b.address, b.client.text())).monospace(),
        );
        lesson_presentation::prose(
            ui,
            &format!(
                "{} · VLAN {} · {} seconds remain",
                b.port,
                b.vlan,
                b.expires.saturating_sub(scope.world.forwarding().now())
            ),
        );
        ui.add_space(h * 0.25);
    }
}
impl LabApp {
    pub(super) fn dhcp_snooping_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::DhcpSnooping { definition }, engine::State::DhcpSnooping { simulation }) =
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
                    record(&mut self.metrics.dhcp_snooping, key, &r, ui);
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
                "Builds on DHCP's initial exchange, VLAN membership and Ethernet source addresses.",
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
                    record(&mut self.metrics.dhcp_snooping, key, &r, ui);
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
            self.metrics.dhcp_snooping.scope = format!("{}:{practice}", source.id);
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.push_id(
                ("dhcp-observer-workspace", &source.id, practice, challenge),
                |ui| {
                    topology(ui, &scope.world);
                    ui.add_enabled_ui(active, |ui| {
                        if practice {
                            lesson_presentation::practice_step(ui, &source, 0);
                        }
                        policy(
                            ui,
                            scope,
                            &mut self.metrics.dhcp_snooping,
                            &mut self.focus_input,
                            &mut self.notice,
                        );
                        if practice {
                            lesson_presentation::practice_step(ui, &source, 1);
                        }
                        let h = ui.text_style_height(&egui::TextStyle::Body);
                        if ui.available_width() >= h * 65. {
                            ui.columns(2, |cols| {
                                replay(
                                    &mut cols[0],
                                    scope,
                                    task,
                                    &mut self.metrics.dhcp_snooping,
                                    &mut self.notice,
                                );
                                leases(&mut cols[1], scope);
                            });
                        } else {
                            replay(
                                ui,
                                scope,
                                task,
                                &mut self.metrics.dhcp_snooping,
                                &mut self.notice,
                            );
                            leases(ui, scope);
                        }
                        if practice {
                            lesson_presentation::practice_step(ui, &source, 2);
                        }
                        let r = ui.add(primary(ui, "Check work"));
                        record(&mut self.metrics.dhcp_snooping, "check", &r, ui);
                        self.metrics.check = Some(r.rect);
                        if r.clicked() {
                            self.notice = scope.check(task).err();
                        }
                        controls::feedback(ui, r.clicked(), |ui| {
                            results(ui, scope, task, &mut self.metrics.dhcp_snooping)
                        });
                        if practice {
                            let worked = controls::collapsing_header(
                                ui,
                                "Worked practice policy · optional",
                            )
                            .show(ui, |ui| {
                                lesson_presentation::worked_rows(ui, &source);
                                let r = ui.add(button("Apply worked example"));
                                record(&mut self.metrics.dhcp_snooping, "fill-example", &r, ui);
                                if r.clicked() {
                                    self.notice = scope.fill(task).err();
                                }
                            });
                            record(
                                &mut self.metrics.dhcp_snooping,
                                "worked-example",
                                &worked.header_response,
                                ui,
                            );
                            let r = ui.add(button("Reset example"));
                            record(&mut self.metrics.dhcp_snooping, "reset-example", &r, ui);
                            if r.clicked() {
                                *scope = work::Scope::new(task);
                                self.notice = None;
                            }
                        }
                        let help = controls::collapsing_header(ui, "Help and simulation scope")
                            .show(ui, |ui| {
                                if !practice {
                                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                                    let r = ui.add(button("Show a hint"));
                                    record(&mut self.metrics.dhcp_snooping, "hint", &r, ui);
                                    if r.clicked() {
                                        a.hints += 1;
                                        self.dirty = true;
                                    }
                                    for hint in source.hints.iter().take(a.hints) {
                                        lesson_presentation::prose(ui, hint);
                                    }
                                    controls::collapsing_header(
                                        ui,
                                        "Worked approach · reveals a solution",
                                    )
                                    .show(ui, |ui| {
                                        lesson_presentation::prose(ui, &source.explanation);
                                    });
                                }
                                lesson_presentation::prose(ui, work::HELP);
                                lesson_presentation::prose(ui, &source.simplifications);
                                for url in &source.sources {
                                    ui.hyperlink_to(url, url);
                                }
                            });
                        record(
                            &mut self.metrics.dhcp_snooping,
                            "help",
                            &help.header_response,
                            ui,
                        );
                    });
                },
            );
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::DhcpSnooping {
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
