//! Virtual Linux terminal and resolver editor; no host command/file dispatch.
use super::*;
use dlms_learning_lab::host_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    /// Total outcomes, failed outcomes, and the rendered summary rectangle.
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn heading(ui: &mut egui::Ui, text: &str) {
    ui.add_space(8.);
    lesson_presentation::rich(ui, RichText::new(text).strong());
}
fn ipv4_facts(ui: &mut egui::Ui, s: &work::Scope) {
    heading(ui, "Supplied network");
    for link in &s.world.profile.links {
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} · LAN {} · carrier {}",
                link.name,
                link.network.text(),
                if link.carrier { "present" } else { "absent" }
            ),
            true,
        );
    }
    for g in &s.world.profile.gateways {
        lesson_presentation::prose(
            ui,
            &format!(
                "Gateway {} on {} · {} · onward to {}",
                g.address,
                g.device,
                if g.available {
                    "Available"
                } else {
                    "Unavailable"
                },
                g.exports
                    .iter()
                    .map(|p| p.text())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    controls::collapsing_header(ui, "Supplied peers and resolver records").show(ui, |ui| {
        lesson_presentation::monospace(
            ui,
            &format!(
                "Known destinations: {}",
                s.world
                    .profile
                    .nodes
                    .iter()
                    .map(|n| format!(
                        "{} ({})",
                        n.address,
                        if n.available {
                            "available"
                        } else {
                            "unavailable"
                        }
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            true,
        );
        for r in &s.world.profile.resolvers {
            lesson_presentation::monospace(
                ui,
                &format!(
                    "Resolver {}: {}",
                    r.address,
                    r.records
                        .iter()
                        .map(|r| format!("{} → {}", r.name, r.address))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
                true,
            );
        }
    });
}
pub(super) fn repairs_both_families(d: &work::Definition, practice: bool) -> bool {
    let rs = if practice {
        &d.example_requirements
    } else {
        &d.requirements
    };
    rs.iter()
        .any(|r| matches!(r, work::Requirement::Address { .. }))
        && rs
            .iter()
            .any(|r| matches!(r, work::Requirement::Ipv6Address { .. }))
}
fn facts(ui: &mut egui::Ui, s: &work::Scope, mixed: bool) {
    if mixed {
        ipv4_facts(ui, s);
    }
    if let Some(facts) = &s.world.profile.ipv6 {
        heading(ui, "Supplied IPv6 network");
        for l in &facts.lans {
            lesson_presentation::monospace(
                ui,
                &format!(
                    "{} · LAN {} · scoped link-local {}",
                    l.device,
                    l.network.text(),
                    l.link_local
                ),
                true,
            );
        }
        for g in &facts.gateways {
            lesson_presentation::prose(
                ui,
                &format!(
                    "Gateway {} on {} · {} · onward to {}",
                    g.address,
                    g.device,
                    if g.available {
                        "Available"
                    } else {
                        "Unavailable"
                    },
                    g.exports
                        .iter()
                        .map(|p| p.text())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
        controls::collapsing_header(ui, "Supplied IPv6 destinations").show(ui, |ui| {
            for n in &facts.nodes {
                lesson_presentation::monospace(
                    ui,
                    &format!(
                        "{} · {}",
                        n.address,
                        if n.available {
                            "Available"
                        } else {
                            "Unavailable"
                        }
                    ),
                    true,
                );
            }
        });
        if !mixed {
            controls::collapsing_header(ui, "Existing IPv4 network · Keep unchanged")
                .show(ui, |ui| ipv4_facts(ui, s));
        }
    } else {
        ipv4_facts(ui, s);
    }
}
fn terminal(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    m: &mut Metrics,
    active: bool,
    mixed: bool,
    events: &mut Vec<work::Event>,
) {
    heading(ui, "Simulated administrator terminal");
    controls::nested_scroll(
        ui,
        egui::ScrollArea::vertical()
            .id_salt("host-output")
            .max_height(270.)
            .min_scrolled_height(130.)
            .stick_to_bottom(true),
        |ui| {
            if s.transcript.is_empty() {
                lesson_presentation::monospace(
                    ui,
                    if mixed {
                        "No commands run yet. Inspect both tables: ip -4 address show and ip -6 address show, then each family's route show."
                    } else if s.world.profile.ipv6.is_some() {
                        "No commands run yet. Start with ip -6 address show or ip -6 route show."
                    } else {
                        "No commands run yet. Start with ip link show or ip -4 address show."
                    },
                    true,
                );
            }
            for text in &s.transcript {
                lesson_presentation::monospace(ui, text, true);
            }
        },
    );
    let mut enter = false;
    controls::form_row(ui, "Command", |ui, width| {
        let r = ui.add_enabled(
            active,
            controls::singleline(ui, &mut s.command)
                .desired_width((width - 2. * f32::from(controls::PAD_X)).max(1.))
                .hint_text("One supported command")
                .font(egui::TextStyle::Monospace)
                .char_limit(512),
        );
        m.fields.push(("command".into(), r.rect, r.id));
        record(m, "command", &r, ui);
        enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
    });
    let r = ui.add_enabled(active, button("Run command"));
    record(m, "run", &r, ui);
    if r.clicked() || enter {
        events.push(work::Event::Command {
            input: s.command.clone(),
        });
    }
}
fn resolver(
    ui: &mut egui::Ui,
    s: &mut work::Scope,
    m: &mut Metrics,
    active: bool,
    events: &mut Vec<work::Event>,
) {
    heading(ui, "Virtual /etc/resolv.conf");
    lesson_presentation::prose(
        ui,
        "One nameserver IPv4-address per line; # comments allowed. Save applies this simulated file only.",
    );
    let r = ui.add_enabled(
        active,
        egui::TextEdit::multiline(&mut s.resolver_draft)
            .font(egui::TextStyle::Monospace)
            .desired_width((ui.available_width() - 12.).max(1.))
            .desired_rows(3)
            .char_limit(512),
    );
    m.fields.push(("resolver".into(), r.rect, r.id));
    record(m, "resolver", &r, ui);
    if r.changed() {
        s.checked = false;
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(active, button("Save resolver"));
        record(m, "save-resolver", &r, ui);
        if r.clicked() {
            events.push(work::Event::Resolver {
                text: s.resolver_draft.clone(),
            });
        }
        let r = ui.add_enabled(
            active && s.resolver_draft != s.resolver_applied,
            button("Discard draft"),
        );
        record(m, "discard", &r, ui);
        if r.clicked() {
            s.resolver_draft = s.resolver_applied.clone();
            s.checked = false;
        }
    });
    if s.resolver_draft != s.resolver_applied {
        lesson_presentation::prose(
            ui,
            "Unapplied resolver changes · lookups still use the saved configuration.",
        );
    }
}
fn probe(ui: &mut egui::Ui, s: &mut work::Scope, m: &mut Metrics, active: bool) {
    heading(ui, "Virtual path test");
    let ipv6 = s.world.profile.ipv6.is_some();
    let input = if ipv6 {
        &mut s.probe_ipv6_input
    } else {
        &mut s.probe_input
    };
    controls::form_row(
        ui,
        if ipv6 {
            "Destination IP address"
        } else {
            "Destination IPv4"
        },
        |ui, width| {
            let r = ui.add_enabled(
                active,
                controls::singleline(ui, input)
                    .desired_width((width - 2. * f32::from(controls::PAD_X)).max(1.))
                    .hint_text(if ipv6 {
                        "IPv4 or IPv6 address"
                    } else {
                        "Dotted-decimal address"
                    })
                    .char_limit(if ipv6 { 64 } else { 32 }),
            );
            m.fields.push(("probe".into(), r.rect, r.id));
            record(m, "probe", &r, ui);
            if r.changed() {
                s.probe_result.clear();
            }
        },
    );
    let r = ui.add_enabled(active, button("Test address · simulator"));
    record(m, "probe-run", &r, ui);
    if r.clicked() {
        if ipv6 {
            s.probe_result = match s.probe_ipv6_input.trim().parse::<std::net::IpAddr>() {
                Ok(std::net::IpAddr::V6(ip)) => { let p=s.world.probe_ipv6(ip); format!("{} · {}",if p.pass{"IPv6 path available"}else{"IPv6 path unavailable"},p.reason) },
                Ok(std::net::IpAddr::V4(ip)) => { let p=s.world.probe(ip); format!("{} · {}",if p.pass{"IPv4 path available"}else{"IPv4 path unavailable"},p.reason) },
                Err(_) => "Enter a literal IPv4 or IPv6 address, without a prefix or device suffix. This path view does not look up names or send packets.".into(),
            };
        } else {
            s.probe_result=match s.probe_input.trim().parse(){Ok(ip)=>{let p=s.world.probe(ip);format!("{} · {}",if p.pass{"Path available"}else{"Path unavailable"},p.reason)},Err(_)=>"Enter four decimal octets, such as 192.0.2.20. This is an IP-path check, not a name lookup.".into()};
        }
    }
    if !s.probe_result.is_empty() {
        lesson_presentation::prose(ui, &s.probe_result);
    }
}
fn apply(next: &mut work::Session, d: &work::Definition, practice: bool, events: Vec<work::Event>) {
    for e in events {
        let prompt = match &e {
            work::Event::Command { input } => format!("# {input}"),
            work::Event::Resolver { .. } => "[Save virtual resolver]".into(),
        };
        let result = next.run(d, &e, practice);
        let s = if practice {
            &mut next.practice
        } else {
            &mut next.assignment
        };
        // The terminal retains actual output and honest parse/refusal diagnostics, not canned success.
        let output = match result {
            Ok(text) => text,
            Err(e) => format!("Not applied · {e}"),
        };
        if s.transcript.len() == 24 {
            s.transcript.remove(0);
        }
        s.transcript.push(if output.is_empty() {
            prompt
        } else {
            format!("{prompt}\n{output}")
        });
        if d.echo.is_none() && d.direct_dns.is_none() {
            s.checked = false;
        }
    }
}
impl LabApp {
    pub(super) fn host_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let engine::Goal::HostNetwork { definition } = &source.goal else {
            return;
        };
        let engine::State::HostNetwork { simulation } = &a.state else {
            return;
        };
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
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(v == view));
                    record(&mut self.metrics.host, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                if definition.initial.profile.ipv6.is_some() {
                    "Builds on Linux IPv4 interface and route practice. IPv6 notation and device scope are explained below."
                } else {
                    "Builds on IPv4 addresses and basic Linux file/terminal work."
                },
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "A separate supplied host demonstrates the repair before your independent assignment.",
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.host, key, &r, ui);
                    if r.clicked() {
                        next.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
                let e = &definition.example_actions[next.example_action];
                let text = match e {
                    work::Event::Command { input } => format!("Worked action · {input}"),
                    work::Event::Resolver { text } => format!("Worked resolver text · {text}"),
                };
                lesson_presentation::monospace(ui, &text, true);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(active, button("Insert worked action"));
                    record(&mut self.metrics.host, "fill-example", &r, ui);
                    if r.clicked() {
                        match e {
                            work::Event::Command { input } => next.practice.command = input.clone(),
                            work::Event::Resolver { text } => {
                                next.practice.resolver_draft = text.clone();
                                next.practice.checked = false;
                            }
                        }
                    }
                    let r = ui.add_enabled(
                        active && next.example_action + 1 < definition.example_actions.len(),
                        button("Next worked action"),
                    );
                    record(&mut self.metrics.host, "next-action", &r, ui);
                    if r.clicked() {
                        next.example_action += 1;
                    }
                });
                lesson_presentation::practice_step(ui, &source, 1);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let mut events = vec![];
            let ipv6 = definition.initial.profile.ipv6.is_some();
            let mixed = repairs_both_families(definition, practice);
            let essential_resolver =
                !ipv6 && (source.id.ends_with("resolver") || source.id.ends_with("handover"));
            let scope = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            if ui.available_width() > 920. * self.scale {
                ui.columns(2, |cols| {
                    terminal(
                        &mut cols[0],
                        scope,
                        &mut self.metrics.host,
                        active,
                        mixed,
                        &mut events,
                    );
                    facts(&mut cols[1], scope, mixed);
                    if definition.echo.is_none() {
                        probe(&mut cols[1], scope, &mut self.metrics.host, active);
                    }
                });
            } else {
                facts(ui, scope, mixed);
                terminal(
                    ui,
                    scope,
                    &mut self.metrics.host,
                    active,
                    mixed,
                    &mut events,
                );
                if definition.echo.is_none() {
                    probe(ui, scope, &mut self.metrics.host, active);
                }
            }
            if !ipv6 && definition.echo.is_none() {
                if essential_resolver {
                    resolver(ui, scope, &mut self.metrics.host, active, &mut events);
                } else {
                    controls::collapsing_header(ui, "Virtual resolver editor (optional)")
                        .show(ui, |ui| {
                            resolver(ui, scope, &mut self.metrics.host, active, &mut events)
                        });
                }
            }
            apply(&mut next, definition, practice, events);
            let s = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check network"
                    },
                ),
            );
            record(&mut self.metrics.host, "check", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                s.checked = true;
            }
            if s.checked {
                let rows = definition.checks(&next, practice);
                let failed = rows.iter().filter(|r| !r.0).count();
                let r = lesson_presentation::rich(
                    ui,
                    RichText::new(if failed == 0 {
                        format!("Passed · all {} outcome checks", rows.len())
                    } else {
                        format!("Needs attention · {failed} of {} checks", rows.len())
                    })
                    .strong(),
                );
                self.metrics.host.result = Some((rows.len(), failed, r.rect));
                for (ok, text) in rows {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!(
                            "{} · {text}",
                            if ok { "Passed" } else { "Needs attention" }
                        ))
                        .color(if ok {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        }),
                    );
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    if ipv6 || definition.echo.is_some() {
                        "Not checked · run your commands, then check the complete network task."
                    } else {
                        "Not checked · run your commands and save resolver changes, then check the complete network task."
                    },
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.host, "reset-example", &r, ui);
                if r.clicked() {
                    next.practice = work::Session::new(definition).practice;
                    next.example_action = 0;
                }
            }
            controls::collapsing_header(ui, "Help and supported commands").show(ui, |ui| {
                lesson_presentation::prose(
                    ui,
                    if definition.initial.profile.ipv6.is_some() {
                        definition.initial.command_help()
                    } else {
                        work::HELP
                    },
                );
                if definition.echo.is_some() {
                    lesson_presentation::prose(ui, dlms_learning_lab::host_echo::HELP);
                }
                if definition.direct_dns.is_some() {
                    lesson_presentation::prose(ui, dlms_learning_lab::host_direct_dns::HELP);
                }
                if !practice {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    let r = ui.add_enabled(
                        active && a.hints < source.hints.len(),
                        button("Show a hint"),
                    );
                    if r.clicked() {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for hint in source.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, hint);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals a solution").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        },
                    );
                }
                for link in &source.sources {
                    ui.hyperlink_to(link, link);
                }
                if let Some(teaching) = lesson_presentation::entry(&source) {
                    for link in &teaching.prerequisite_sources {
                        ui.hyperlink_to(link, link);
                    }
                }
            });
        }
        if next != before && active {
            match definition.bound(&next) {
                Ok(()) => {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    a.state = engine::State::HostNetwork {
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
