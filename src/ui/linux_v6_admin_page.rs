//! Native IPv6 configuration terminal/editor. No host command execution.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    linux_v6_admin_source as binding, linux_v6_admin_workspace as work, linux_v6_saved_settings,
};
fn apply(s: &mut work::Scope, t: &work::Task, a: work::Action, n: &mut Option<String>) {
    if let Err(e) = s.apply(t, &a) {
        *n = Some(e);
    }
}
fn topology(ui: &mut egui::Ui, s: &work::Scope) -> egui::Response {
    let c = s
        .world
        .host
        .configuration
        .ipv6
        .as_ref()
        .expect("validated IPv6 source");
    let rows = c
        .interfaces
        .iter()
        .map(|i| {
            [
                format!("{} · primary", i.device),
                i.address.map_or("No primary address".into(), |a| {
                    format!("{}/{}", a.address, a.prefix)
                }),
                format!(
                    "Link-local {} · {}",
                    s.world
                        .host
                        .profile
                        .ipv6
                        .as_ref()
                        .and_then(|p| p.lans.iter().find(|l| l.device == i.device))
                        .map_or("Not supplied".into(), |l| l.link_local.to_string()),
                    i.device
                ),
            ]
        })
        .collect::<Vec<_>>();
    let role = if s.world.settings.global_forwarding {
        "Transit enabled"
    } else {
        "Transit disabled"
    };
    let mut cards = rows
        .iter()
        .map(|r| [r[0].as_str(), r[1].as_str(), r[2].as_str()])
        .collect::<Vec<_>>();
    cards.insert(
        cards.len().div_ceil(2),
        [
            "Linux IPv6 router",
            role,
            "Routes and settings are editable",
        ],
    );
    controls::topology_chain(
        ui,
        &cards,
        "Current simulated configuration · a link-local next hop is identified by both its address and its interface.",
    )
}
fn worked(ui: &mut egui::Ui, source: &Scenario, d: &work::Definition) {
    for a in &d.worked {
        match a {
            work::Action::Command { input } => {
                lesson_presentation::monospace(ui, input, true);
                if let Some(c) = source
                    .guide
                    .iter()
                    .flat_map(|g| &g.commands)
                    .find(|c| c.command == *input)
                {
                    lesson_presentation::prose(ui, &c.explanation);
                }
            }
            work::Action::EditFile { text } => {
                ui.strong("Worked example file");
                lesson_presentation::monospace(ui, text, true);
            }
            work::Action::SaveFile => {
                lesson_presentation::prose(
                    ui,
                    "Choose Save virtual file; current settings stay unchanged until loaded.",
                );
            }
            work::Action::Observe { message } => {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "Choose {}, then Observe supplied message.",
                        d.example.messages[*message].label
                    ),
                );
            }
            work::Action::RunRestart => {
                lesson_presentation::prose(
                    ui,
                    "Choose Run saved-settings test; inspect its actual results.",
                );
            }
            work::Action::Check => {
                lesson_presentation::prose(
                    ui,
                    "Choose Check example to verify the complete assignment.",
                );
            }
            _ => {}
        }
        ui.add_space(6.);
    }
}
fn editor(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    let label = ui.strong("IPv6 startup settings · simulated file");
    lesson_presentation::monospace(ui, linux_v6_saved_settings::FILE, true);
    let mut text = s.file_editor.clone();
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 7. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scrolled) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("v6-admin-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt("v6-admin-file")
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .desired_width(
                            (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(1.),
                        )
                        .desired_rows(7)
                        .char_limit(4096),
                )
            })
    });
    m.controls
        .push(("editor-viewport".into(), outer, ui.clip_rect()));
    let r = scrolled.inner.labelled_by(label.id);
    m.fields.push(("editor".into(), r.rect, r.id));
    m.focus.push(("editor".into(), r.id));
    m.controls.push((
        "editor".into(),
        r.rect.intersect(scrolled.inner_rect),
        ui.clip_rect(),
    ));
    if r.changed() {
        apply(s, t, work::Action::EditFile { text }, n);
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save virtual file"));
        m.record(ui, "save", &r);
        if r.clicked() {
            apply(s, t, work::Action::SaveFile, n);
        }
        let r = ui.label(if s.file_editor == s.saved_file {
            "Saved · load separately"
        } else {
            "Unsaved changes"
        });
        m.record(ui, "file-state", &r);
    });
    let r = ui.add(button("Run saved-settings test"));
    m.record(ui, "restart", &r);
    if r.clicked() {
        apply(s, t, work::Action::RunRestart, n);
    }
    if let Some(trial) = &s.last_restart {
        let pass = trial.checks.iter().filter(|c| c.passed).count();
        let r = ui.strong(format!(
            "Actual saved-settings test · {pass}/{} outcomes pass",
            trial.checks.len()
        ));
        m.record(ui, "restart-result", &r);
        for c in trial.checks.iter().filter(|c| !c.passed) {
            lesson_presentation::prose(ui, &format!("Needs work · {}: {}", c.label, c.detail));
        }
        lesson_presentation::prose(
            ui,
            "This test resets modeled flags and learned routes. The supplied network is already persistent; it does not reboot an operating system.",
        );
    }
}
fn console(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    m: &mut Metrics,
    n: &mut Option<String>,
    phase: (bool, &mut bool),
) {
    let (practice, focus) = phase;
    ui.strong("Router terminal · simulated commands");
    if practice {
        let commands = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            let key = egui::Id::new(("v6-admin-choice", &source.id));
            let mut at = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, w| {
                let r = controls::dropdown(ui, "v6-admin-command-choice", w)
                    .selected_text(&commands[at].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        for (i, c) in commands.iter().enumerate() {
                            ui.selectable_value(&mut at, i, &c.command);
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "command-choice", &r.response);
            });
            lesson_presentation::prose(ui, &commands[at].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                apply(
                    s,
                    t,
                    work::Action::Input {
                        text: commands[at].command.clone(),
                    },
                    n,
                );
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, at));
        }
    }
    let h = ui.text_style_height(&egui::TextStyle::Monospace)
        * s.output.lines().count().clamp(4, 10) as f32;
    let out=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{ui.strong("Actual command results");controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("v6-admin-output").auto_shrink([false,false]).min_scrolled_height(h).max_height(h).stick_to_bottom(true),|ui|{if s.output.is_empty(){lesson_presentation::prose(ui,"No command has run. Inspect the supplied addresses, routes or settings to begin.");}else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}});});
    m.record(ui, "output-viewport", &out.response);
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("v6-admin-input", &source.id, practice)),
            512,
            ui.is_enabled(),
            "Command · Enter runs",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(text) = r.value {
            apply(s, t, work::Action::Input { text }, n);
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
                "Keep one literal command within 512 bytes; previous input retained.",
            );
        }
    };
    let bw = controls::button_width(ui, "Run command");
    let clicked = if ui.available_width()
        > bw + ui.text_style_height(&egui::TextStyle::Body) * 10. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let width = (ui.available_width() - bw - ui.spacing().item_spacing.x).max(1.);
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
        apply(
            s,
            t,
            work::Action::Command {
                input: s.input.clone(),
            },
            n,
        );
    }
}
fn trials(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    if !t.messages.is_empty() {
        ui.strong("Supplied router advertisements");
        let mut at = s.selected_message;
        controls::form_row(ui, "Message", |ui, w| {
            let r = controls::dropdown(ui, "v6-admin-message", w)
                .selected_text(&t.messages[at].label)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    for (i, c) in t.messages.iter().enumerate() {
                        let option = ui.selectable_value(&mut at, i, &c.label);
                        m.record(ui, &format!("message:{i}"), &option);
                    }
                });
            controls::retain_dropdown_focus(ui, &r.response);
            m.record(ui, "message", &r.response);
        });
        if at != s.selected_message {
            apply(s, t, work::Action::SelectMessage { message: at }, n);
        }
        let a = &t.messages[s.selected_message].advertisement;
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} on {} · router lifetime {}",
                a.gateway, a.device, a.router_lifetime
            ),
            true,
        );
        let r = ui.add(button("Observe supplied message"));
        m.record(ui, "observe", &r);
        if r.clicked() {
            apply(
                s,
                t,
                work::Action::Observe {
                    message: s.selected_message,
                },
                n,
            );
        }
        lesson_presentation::prose(
            ui,
            "Simulator observation of an already-received message. Actual acceptance appears in the command results.",
        );
    }
    ui.strong("Supplied transit cases");
    let mut at = s.selected_frame;
    controls::form_row(ui, "Transit case", |ui, w| {
        let r = controls::dropdown(ui, "v6-admin-frame", w)
            .selected_text(&t.frames[at].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in t.frames.iter().enumerate() {
                    let option = ui.selectable_value(&mut at, i, &c.label);
                    m.record(ui, &format!("case:{i}"), &option);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "case", &r.response);
    });
    if at != s.selected_frame {
        apply(s, t, work::Action::SelectFrame { frame: at }, n);
    }
    let f = &t.frames[s.selected_frame];
    lesson_presentation::monospace(
        ui,
        &format!(
            "{} → {} · ingress {}",
            f.header.source, f.header.destination, f.ingress
        ),
        true,
    );
    let r = ui.add(button("Run transit test"));
    m.record(ui, "run-traffic", &r);
    if r.clicked() {
        apply(s, t, work::Action::RunFrame, n);
    }
    if let Some(trial) = &s.last_trial {
        let r = lesson_presentation::prose(
            ui,
            &format!(
                "Actual transit result · {}\n{}",
                if trial.result.passed {
                    "Passed"
                } else {
                    "Needs work"
                },
                trial.result.detail
            ),
        );
        m.record(ui, "traffic-result", &r);
    } else {
        lesson_presentation::prose(ui, "Not run for the current selection/configuration.");
    }
}
impl LabApp {
    pub(super) fn v6_admin_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (
            engine::Goal::LinuxV6Administration { definition },
            engine::State::LinuxV6Administration { simulation },
        ) = (&source.goal, &original.state)
        else {
            return;
        };
        if let Err(e) = definition.bound(simulation) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let first = egui::Id::new(("v6-admin-first", &source.id, original.visited));
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
                    self.metrics.v6_admin.record(ui, key, &r);
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
                .v6_admin
                .record(ui, "introduction-content", &intro.objective);
            lesson_presentation::worked(
                ui,
                "Separate worked example · these values describe the example, not actual run results.",
            );
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics.v6_admin.record(ui, "worked", &r.response);
            controls::collapsing_header(ui, "Complete worked sequence · example only")
                .show(ui, |ui| worked(ui, &source, definition));
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.v6_admin.record(ui, "try-example", &r);
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
                self.metrics.v6_admin.record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
 let r=topology(ui,s);self.metrics.v6_admin.record(ui,"topology",&r);
 let facts=t.initial.host.profile.ipv6.as_ref().expect("validated supplied network");
 for lan in &facts.lans {let peers=facts.nodes.iter().filter(|p|lan.network.contains(p.address)).map(|p|p.address.to_string()).collect::<Vec<_>>().join(", ");lesson_presentation::monospace(ui,&format!("Supplied {} network {}/{} · known peers {}",lan.device,lan.network.address,lan.network.length,peers),true);}
 controls::collapsing_header(ui,"Supplied next hops and remote hosts").show(ui,|ui|{for g in &facts.gateways {lesson_presentation::monospace(ui,&format!("{} on {} · {} · onward {}",g.address,g.device,if g.available{"Available"}else{"Unavailable"},g.exports.iter().map(|p|p.text()).collect::<Vec<_>>().join(", ")),true);}lesson_presentation::monospace(ui,&format!("Known supplied hosts: {}",facts.nodes.iter().map(|p|p.address.to_string()).collect::<Vec<_>>().join(", ")),true);});

 if practice{lesson_presentation::practice_step(ui,&source,0);}
 if t.file.is_some(){if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.{ui.columns(2,|cols|{editor(&mut cols[0],t,s,&mut self.metrics.v6_admin,&mut self.notice);console(&mut cols[1],t,s,&source,&mut self.metrics.v6_admin,&mut self.notice,(practice,&mut self.focus_input));});}else{editor(ui,t,s,&mut self.metrics.v6_admin,&mut self.notice);console(ui,t,s,&source,&mut self.metrics.v6_admin,&mut self.notice,(practice,&mut self.focus_input));}}else{console(ui,t,s,&source,&mut self.metrics.v6_admin,&mut self.notice,(practice,&mut self.focus_input));}
 if practice{lesson_presentation::practice_step(ui,&source,1);}trials(ui,t,s,&mut self.metrics.v6_admin,&mut self.notice);if practice{lesson_presentation::practice_step(ui,&source,2);}
 let r=ui.add(primary(ui,if practice{"Check example"}else{"Check work"}));self.metrics.v6_admin.record(ui,"check",&r);if r.clicked(){apply(s,t,work::Action::Check,&mut self.notice);}
 if s.checked{let checks=s.checks(t);let pass=checks.iter().filter(|c|c.passed).count();let r=ui.strong(if pass==checks.len(){if practice{"Example checks passed".into()}else{"Assignment complete".into()}}else{format!("Not complete · {pass}/{} outcomes pass",checks.len())});self.metrics.v6_admin.result=Some((pass,checks.len(),r.rect));self.metrics.v6_admin.record(ui,"result",&r);for c in checks.iter().filter(|c|!c.passed){let r=lesson_presentation::prose(ui,&format!("Needs work · {}\n{}",c.label,c.detail));self.metrics.v6_admin.record(ui,"failed-case",&r);}controls::collapsing_header(ui,"Inspect passing checks").show(ui,|ui|{for c in checks.iter().filter(|c|c.passed){lesson_presentation::prose(ui,&format!("Passed · {}\n{}",c.label,c.detail));}});if pass==checks.len(){lesson_presentation::prose(ui,if practice{"The example meets its address, route, role and supplied test requirements. Apply the same reasoning to the separate assignment."}else{&source.explanation});}}
 });
            controls::collapsing_header(ui, "Optional help and supported commands").show(
                ui,
                |ui| {
                    if practice {
                        controls::collapsing_header(ui, "Show worked answer · example only")
                            .show(ui, |ui| worked(ui, &source, definition));
                    } else {
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
                        engine::State::LinuxV6Administration {
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn v6_admin_topology_renders_bounded_link_counts_and_actual_supplied_link_local_values() {
        let source = serde_json::from_str::<pack::Pack>(include_str!(
            "../../scenarios/linux-v6-admin-lessons-v1.json"
        ))
        .unwrap()
        .scenarios
        .remove(0);
        let engine::State::LinuxV6Administration { simulation } = source.initial else {
            panic!()
        };
        for count in [1, 2, 8] {
            for (width, scale) in [(1450., 1.), (760., 2.)] {
                // Renderer geometry fixture only; source/model validation is checked separately.
                let mut scope = simulation.assignment.clone();
                let c = scope.world.host.configuration.ipv6.as_mut().unwrap();
                let prototype = c.interfaces[0].clone();
                c.interfaces.resize(count, prototype);
                for (i, port) in c.interfaces.iter_mut().enumerate() {
                    port.device = format!("port{i}");
                }
                let ctx = egui::Context::default();
                ctx.set_zoom_factor(scale);
                theme::apply(&ctx, "dlms-navy");
                let input = egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(width, 12000.),
                    )),
                    ..Default::default()
                };
                let _ = ctx.run(input, |ctx| {
                    controls::configure(ctx);
                    egui::CentralPanel::default().show(ctx, |ui| {
                        let r = topology(ui, &scope);
                        assert!(
                            r.rect.is_finite() && r.rect.right() <= ui.clip_rect().right() + 1.
                        );
                    });
                });
            }
        }
        let scope = &simulation.assignment;
        let facts = scope.world.host.profile.ipv6.as_ref().unwrap();
        assert_eq!(facts.lans[0].link_local.to_string(), "fe80::a");
    }
}
