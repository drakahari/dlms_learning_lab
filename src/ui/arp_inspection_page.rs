//! Native ARP-policy work; no terminal or host-network execution.
use super::*;
use dlms_learning_lab::{arp_inspection as core, arp_inspection_workspace as work};
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
fn field(
    ui: &mut egui::Ui,
    m: &mut Metrics,
    focus: &mut bool,
    key: &str,
    value: &mut String,
    hint: &str,
    width: f32,
) {
    let r = ui.add(
        controls::singleline_fitting(ui, value, width)
            .id(egui::Id::new(("arp-policy-field", &m.scope, key)))
            .hint_text(hint),
    );
    if *focus {
        r.request_focus();
        *focus = false;
    }
    m.fields.push((key.into(), r.rect, r.id, ui.clip_rect()));
}
fn basic(ui: &mut egui::Ui, s: &mut work::Scope, m: &mut Metrics, focus: &mut bool) {
    lesson_presentation::prose(
        ui,
        "ARP checks run in the listed VLANs. A trusted incoming port bypasses them; DHCP-server trust is separate.",
    );
    controls::form_row(ui, "Inspect ARP in VLANs", |ui, w| {
        field(
            ui,
            m,
            focus,
            "vlans",
            &mut s.draft.vlans,
            "Comma-separated VLAN IDs",
            w.min(440.),
        )
    });
    for p in &mut s.draft.ports {
        controls::form_row(ui, &p.port, |ui, _| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
            let r = ui.checkbox(&mut p.trusted, "Bypass ARP inspection");
            record(m, &format!("trust-{}", p.port), &r, ui);
        });
    }
}
fn comparison(ui: &mut egui::Ui, s: &mut work::Scope, m: &mut Metrics) {
    lesson_presentation::prose(
        ui,
        "These optional checks compare packet fields. The destination comparison applies to replies, not requests.",
    );
    for (key, label, v) in [
        (
            "check-source",
            "Compare Ethernet source and ARP sender MACs",
            &mut s.draft.checks.source_mac,
        ),
        (
            "check-destination",
            "Compare Ethernet destination and ARP reply target MACs",
            &mut s.draft.checks.destination_mac,
        ),
    ] {
        let r = ui.checkbox(v, label);
        record(m, key, &r, ui);
    }
    let c=controls::collapsing_header(ui,"IP field check · optional").show(ui,|ui|{
        lesson_presentation::prose(ui,"This check rejects zero, broadcast and multicast sender IPs, and those target IPs in replies. It does not check the target IP in a request.");
        let r=ui.checkbox(&mut s.draft.checks.ip,"Validate sender IPs and reply target IPs");record(m,"check-ip",&r,ui);
    });
    record(m, "ip-comparison", &c.header_response, ui);
}
fn action(ui: &mut egui::Ui, m: &mut Metrics, key: &str, value: &mut core::Action, width: f32) {
    let c = controls::dropdown(ui, key, width)
        .selected_text(match value {
            core::Action::Permit => "Permit",
            core::Action::Deny => "Deny",
        })
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (v, label, name) in [
                (core::Action::Permit, "Permit", "permit"),
                (core::Action::Deny, "Deny", "deny"),
            ] {
                let r = ui.selectable_value(value, v, label);
                record(m, &format!("{key}-{name}"), &r, ui);
            }
        });
    record(m, key, &c.response, ui);
}
fn lists(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    focus: &mut bool,
) {
    lesson_presentation::prose(
        ui,
        "Rules match the ARP sender's IP and MAC together, from top to bottom. A matching deny takes precedence over a DHCP record.",
    );
    let h = ui.text_style_height(&egui::TextStyle::Body);
    let mut remove_list = None;
    for (i, f) in s.draft.filters.iter_mut().enumerate() {
        ui.push_id(("arp-address-list", i), |ui| {
            lesson_presentation::rich(
                ui,
                RichText::new(format!("Address list {}", i + 1)).strong(),
            );
            controls::form_row(ui, "List VLAN", |ui, w| {
                field(
                    ui,
                    m,
                    focus,
                    &format!("list-{i}-vlan"),
                    &mut f.vlan,
                    "VLAN number",
                    w.min(h * 10.),
                )
            });
            let r = ui.checkbox(
                &mut f.static_only,
                "Require a matching list rule (skip DHCP records)",
            );
            record(m, &format!("list-{i}-static"), &r, ui);
            let wide = ui.available_width() >= h * 48.;
            let width = (ui.available_width() - 3. * controls::GAP).max(0.);
            let cols = [width * 0.18, width * 0.27, width * 0.35, width * 0.20];
            if wide {
                ui.horizontal(|ui| {
                    for (label, w) in ["Action", "Sender IP", "Sender MAC", ""]
                        .into_iter()
                        .zip(cols)
                    {
                        controls::label(ui, RichText::new(label).strong(), w);
                    }
                });
            }
            let mut remove_rule = None;
            for (j, row) in f.rules.iter_mut().enumerate() {
                let key = format!("list-{i}-rule-{j}");
                if wide {
                    ui.horizontal(|ui| {
                        controls::cell(ui, cols[0], |ui| {
                            action(ui, m, &format!("{key}-action"), &mut row.action, cols[0])
                        });
                        controls::cell(ui, cols[1], |ui| {
                            field(
                                ui,
                                m,
                                focus,
                                &format!("{key}-ip"),
                                &mut row.ip,
                                "Dotted-decimal IP",
                                cols[1],
                            )
                        });
                        controls::cell(ui, cols[2], |ui| {
                            field(
                                ui,
                                m,
                                focus,
                                &format!("{key}-mac"),
                                &mut row.mac,
                                "aa:bb:cc:dd:ee:ff",
                                cols[2],
                            )
                        });
                        controls::cell(ui, cols[3], |ui| {
                            let r = ui.add(button("Remove rule"));
                            record(m, &format!("{key}-remove"), &r, ui);
                            if r.clicked() {
                                remove_rule = Some(j);
                            }
                        });
                    });
                } else {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!("Rule {}", j + 1)).strong(),
                    );
                    controls::form_row(ui, "Action", |ui, w| {
                        action(ui, m, &format!("{key}-action"), &mut row.action, w)
                    });
                    controls::form_row(ui, "Sender IP", |ui, w| {
                        field(
                            ui,
                            m,
                            focus,
                            &format!("{key}-ip"),
                            &mut row.ip,
                            "Dotted-decimal IP",
                            w,
                        )
                    });
                    controls::form_row(ui, "Sender MAC", |ui, w| {
                        field(
                            ui,
                            m,
                            focus,
                            &format!("{key}-mac"),
                            &mut row.mac,
                            "aa:bb:cc:dd:ee:ff",
                            w,
                        )
                    });
                    let r = ui.add(button("Remove rule"));
                    record(m, &format!("{key}-remove"), &r, ui);
                    if r.clicked() {
                        remove_rule = Some(j);
                    }
                }
            }
            if let Some(j) = remove_rule {
                f.rules.remove(j);
            }
            if f.rules.len() > 1 {
                let mut move_rule = None;
                let c = controls::collapsing_header(ui, "Rule order · optional").show(ui, |ui| {
                    for (j, row) in f.rules.iter().enumerate() {
                        lesson_presentation::prose(
                            ui,
                            &format!("Rule {} · {} · {}", j + 1, row.ip, row.mac),
                        );
                        ui.horizontal_wrapped(|ui| {
                            for (name, label, enabled, target) in [
                                ("earlier", "Move earlier", j > 0, j.saturating_sub(1)),
                                ("later", "Move later", j + 1 < f.rules.len(), j + 1),
                            ] {
                                let r = ui.add_enabled(enabled, button(label));
                                record(m, &format!("list-{i}-rule-{j}-{name}"), &r, ui);
                                if r.clicked() {
                                    move_rule = Some((j, target));
                                }
                            }
                        });
                    }
                });
                record(m, &format!("list-{i}-order"), &c.header_response, ui);
                if let Some((from, to)) = move_rule {
                    f.rules.swap(from, to);
                }
            }
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(f.rules.len() < 16, button("Add rule"));
                record(m, &format!("list-{i}-add-rule"), &r, ui);
                if r.clicked() {
                    f.rules.push(work::RuleDraft {
                        action: core::Action::Permit,
                        ip: String::new(),
                        mac: String::new(),
                    });
                }
                let r = ui.add(button("Remove list"));
                record(m, &format!("list-{i}-remove"), &r, ui);
                if r.clicked() {
                    remove_list = Some(i);
                }
            });
            ui.separator();
        });
    }
    if let Some(i) = remove_list {
        s.draft.filters.remove(i);
    }
    let r = ui.add_enabled(s.draft.filters.len() < 16, button("Add address list"));
    record(m, "add-list", &r, ui);
    if r.clicked() {
        let vlan = t
            .initial
            .dhcp()
            .baseline()
            .fabric()
            .switches
            .values()
            .next()
            .unwrap()
            .vlans
            .iter()
            .find(|v| {
                !s.draft
                    .filters
                    .iter()
                    .any(|f| f.vlan.trim() == v.to_string())
            })
            .copied()
            .unwrap_or(10);
        s.draft.filters.push(work::FilterDraft {
            vlan: vlan.to_string(),
            static_only: false,
            rules: vec![],
        });
    }
}
fn policy(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    focus: &mut bool,
    notice: &mut Option<String>,
) {
    let before = s.draft.clone();
    let needs_basic = t.initial.config().vlans != t.reference.vlans
        || t.initial.config().ports != t.reference.ports;
    let needs_lists = !t.initial.config().filters.is_empty() || !t.reference.filters.is_empty();
    let needs_comparison = t.initial.config().checks != t.reference.checks;
    ui.scope(|ui| {
        ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Wrap);
        if needs_basic {
            basic(ui, s, m, focus);
        } else {
            let c = controls::collapsing_header(ui, "VLAN and port trust · optional")
                .show(ui, |ui| basic(ui, s, m, focus));
            record(m, "basic", &c.header_response, ui);
        }
        if needs_lists {
            lists(ui, s, t, m, focus);
        } else {
            let c = controls::collapsing_header(ui, "Static address lists · optional")
                .show(ui, |ui| lists(ui, s, t, m, focus));
            record(m, "lists", &c.header_response, ui);
        }
        if needs_comparison {
            comparison(ui, s, m);
        } else {
            let c = controls::collapsing_header(ui, "Extra field comparisons · optional")
                .show(ui, |ui| comparison(ui, s, m));
            record(m, "comparison", &c.header_response, ui);
        }
    });
    if s.draft != before {
        s.checked = false;
        s.output.clear();
    }
    let r = ui.add(button("Apply ARP policy"));
    record(m, "apply", &r, ui);
    if r.clicked() {
        *notice = s.apply().err();
    }
    if let Some(e) = notice.as_deref() {
        lesson_presentation::rich(ui, RichText::new(e).color(palette(ui).error));
    }
    lesson_presentation::prose(
        ui,
        if s.applied() {
            "Current ARP policy applied."
        } else {
            "Changes are not applied yet."
        },
    );
}
fn replay(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    t: &work::Task,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let before = s.sample;
    let width = ui
        .available_width()
        .min((ui.clip_rect().right() - ui.next_widget_position().x).max(0.));
    let c = controls::dropdown(ui, "arp-sample", width)
        .selected_text(&t.cases[s.sample].label)
        .show_ui(ui, |ui| {
            controls::fit_popup_contents(ui);
            for (i, case) in t.cases.iter().enumerate() {
                let r = ui.selectable_value(&mut s.sample, i, &case.label);
                record(m, &format!("sample-{i}"), &r, ui);
            }
        });
    record(m, "sample", &c.response, ui);
    if s.sample != before {
        s.checked = false;
        s.output.clear();
    }
    if let Some(work::Step::Arp { packet, .. }) = t.cases[s.sample]
        .steps
        .iter()
        .find(|s| matches!(s, work::Step::Arp { .. }))
    {
        let compare = t.initial.config().checks != t.reference.checks;
        let mut fields = vec![
            ("ARP sender MAC", packet.sender_mac.text()),
            ("ARP sender IP", packet.sender_ip.to_string()),
            ("ARP target IP", packet.target_ip.to_string()),
        ];
        if compare {
            fields.insert(0, ("Ethernet source MAC", packet.ethernet_source.text()));
            fields.extend([
                (
                    "Ethernet destination MAC",
                    packet.ethernet_destination.text(),
                ),
                ("ARP target MAC", packet.target_mac.text()),
            ]);
        }
        for (label, value) in fields {
            controls::form_row(ui, label, |ui, _| {
                lesson_presentation::rich(ui, RichText::new(value).monospace());
            });
        }
        if !compare {
            controls::collapsing_header(ui, "Ethernet header fields · optional").show(ui, |ui| {
                for (label, value) in [
                    ("Ethernet source MAC", packet.ethernet_source.text()),
                    (
                        "Ethernet destination MAC",
                        packet.ethernet_destination.text(),
                    ),
                    ("ARP target MAC", packet.target_mac.text()),
                ] {
                    controls::form_row(ui, label, |ui, _| {
                        lesson_presentation::rich(ui, RichText::new(value).monospace());
                    });
                }
            });
        }
        lesson_presentation::prose(
            ui,
            if packet.operation == core::Operation::Request {
                "ARP request · asks who has the target IP. Its target MAC can be unknown."
            } else {
                "ARP reply · supplies the sender's address claim to the target."
            },
        );
    }
    lesson_presentation::prose(
        ui,
        "Replay starts a fresh sample network and replaces earlier observations. Your policy draft stays in place.",
    );
    let r = ui.add_enabled(s.applied(), button("Replay sample"));
    record(m, "replay", &r, ui);
    if r.clicked() {
        *notice = s.run_sample(t).err();
    }
    lesson_presentation::rich(ui, RichText::new("Actual replay results").strong());
    let height = ui.text_style_height(&egui::TextStyle::Body) * 8.;
    let (rect, _) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::vertical()
                .id_salt("arp-replay-output")
                .min_scrolled_height(height)
                .max_height(height)
                .auto_shrink([false, false])
                .stick_to_bottom(true),
            |ui| {
                if s.output.is_empty() {
                    lesson_presentation::prose(ui, "Not run for this policy and sample yet.");
                }
                for line in &s.output {
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
    m.replay_output = Some(rect);
    lesson_presentation::rich(
        ui,
        RichText::new(if s.output.is_empty() {
            "Earlier DHCP observations"
        } else {
            "Current DHCP address records"
        })
        .strong(),
    );
    if s.world.dhcp().bindings().is_empty() {
        lesson_presentation::prose(ui, "No current record in this sample state.");
    }
    for b in s.world.dhcp().bindings() {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {} · VLAN {}, {} · {} seconds remain",
                b.address,
                b.client.text(),
                b.vlan,
                b.port,
                b.expires.saturating_sub(s.world.dhcp().forwarding().now())
            ))
            .monospace(),
        );
    }
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &work::Task, m: &mut Metrics) {
    if !s.checked || !s.applied() {
        lesson_presentation::prose(
            ui,
            if s.applied() {
                "Not checked · compare the policy with every traffic requirement when ready."
            } else {
                "Apply current ARP policy before checking."
            },
        );
        return;
    }
    let reports = t.reports(&s.world);
    let passed = reports.iter().filter(|r| r.passed()).count();
    let r = lesson_presentation::rich(
        ui,
        RichText::new(if passed == reports.len() {
            "All ARP traffic checks match"
        } else {
            "ARP policy needs attention"
        })
        .strong()
        .color(if passed == reports.len() {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.result = Some((passed, reports.len(), r.rect));
    for (i, r) in reports.iter().enumerate() {
        lesson_presentation::rich(
            ui,
            RichText::new(format!(
                "{} · {}",
                if r.passed() { "Matches" } else { "Needs work" },
                r.label.as_deref().unwrap_or("Supplied workspace")
            ))
            .strong(),
        );
        if let Some(first) = r.failures.first() {
            lesson_presentation::prose(ui, first);
        }
        if r.failures.len() > 1 {
            ui.push_id(("arp-case-detail", i), |ui| {
                controls::collapsing_header(ui, "More observations in this case").show(ui, |ui| {
                    for e in r.failures.iter().skip(1) {
                        lesson_presentation::prose(ui, e);
                    }
                });
            });
        }
    }
}
impl LabApp {
    pub(super) fn arp_inspection_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
            engine::Goal::ArpInspection { definition },
            engine::State::ArpInspection { simulation },
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
                    record(&mut self.metrics.arp_inspection, key, &r, ui);
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
                "Builds on Ethernet source addresses and DHCP snooping's confirmed address records.",
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
                    record(&mut self.metrics.arp_inspection, key, &r, ui);
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
            self.metrics.arp_inspection.scope = format!("{}:{practice}", source.id);
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.push_id(("arp-inspection-workspace",&source.id,practice,challenge),|ui|{
                super::dhcp_snooping_page::topology_with_caption(ui,scope.world.dhcp(),"ARP trust bypasses inspection on incoming traffic; it is independent of DHCP-server trust.");
                ui.add_enabled_ui(active,|ui|{
                    if practice{lesson_presentation::practice_step(ui,&source,0);}
                    policy(ui,scope,task,&mut self.metrics.arp_inspection,&mut self.focus_input,&mut self.notice);
                    if practice{lesson_presentation::practice_step(ui,&source,1);}
                    replay(ui,scope,task,&mut self.metrics.arp_inspection,&mut self.notice);
                    if practice{lesson_presentation::practice_step(ui,&source,2);}
                    lesson_presentation::prose(ui,"Check work tests every traffic case with its own sample network. It leaves your displayed observations unchanged.");
                    let r=ui.add(primary(ui,"Check work"));record(&mut self.metrics.arp_inspection,"check",&r,ui);self.metrics.check=Some(r.rect);
                    if r.clicked(){self.notice=scope.check(task).err();}
                    controls::feedback(ui,r.clicked(),|ui|results(ui,scope,task,&mut self.metrics.arp_inspection));
                    if practice{
                        let c=controls::collapsing_header(ui,"Worked practice policy · optional").show(ui,|ui|{
                            lesson_presentation::worked_rows(ui,&source);let r=ui.add(button("Apply worked example"));record(&mut self.metrics.arp_inspection,"fill-example",&r,ui);if r.clicked(){self.notice=scope.fill(task).err();}
                        });record(&mut self.metrics.arp_inspection,"worked-example",&c.header_response,ui);
                        let r=ui.add(button("Reset example"));record(&mut self.metrics.arp_inspection,"reset-example",&r,ui);if r.clicked(){*scope=work::Scope::new(task);self.notice=None;}
                    }
                    let c=controls::collapsing_header(ui,"Help and simulation limits").show(ui,|ui|{
                        if !practice{
                            let a=self.progress.attempts.get_mut(&source.id).unwrap();let r=ui.add(button("Show a hint"));record(&mut self.metrics.arp_inspection,"hint",&r,ui);if r.clicked(){a.hints+=1;self.dirty=true;}
                            for h in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,h);}
                            controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{lesson_presentation::prose(ui,&source.explanation);});
                        }
                        lesson_presentation::prose(ui,work::HELP);lesson_presentation::prose(ui,&source.simplifications);for url in &source.sources{ui.hyperlink_to(url,url);}
                    });record(&mut self.metrics.arp_inspection,"help",&c.header_response,ui);
                });
            });
        }
        if candidate != before && active {
            match definition.bound(&candidate) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::ArpInspection {
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
