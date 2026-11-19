//! Native editor/terminal for the provided IPv6 firewall loader; no host execution.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    linux_v6_startup as core, linux_v6_startup_source as binding,
    linux_v6_startup_workspace as work,
};
fn apply(s: &mut work::Scope, t: &work::Task, a: work::Action, n: &mut Option<String>) {
    if let Err(e) = s.apply(t, &a) {
        *n = Some(e);
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
    ui.strong("Firewall terminal · simulated commands");
    if practice {
        let commands = source
            .guide
            .iter()
            .flat_map(|g| &g.commands)
            .collect::<Vec<_>>();
        if !commands.is_empty() {
            let key = egui::Id::new(("v6-startup-choice", &source.id));
            let mut at = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, w| {
                let r = controls::dropdown(ui, "v6-startup-command-choice", w)
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
    let out=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{ui.strong("Actual command results");controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("v6-startup-output").auto_shrink([false,false]).min_scrolled_height(h).max_height(h).stick_to_bottom(true),|ui|{if s.output.is_empty(){lesson_presentation::prose(ui,"No command has run. Inspect the current rules, saved file and provided loader to begin.");}else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}});});
    m.record(ui, "output-viewport", &out.response);
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("v6-startup-input", &source.id, practice)),
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
                    "Choose Save virtual file; load its rules separately.",
                );
            }
            work::Action::RunStartup => {
                lesson_presentation::prose(
                    ui,
                    "Choose Run startup test and inspect its actual policy results.",
                );
            }
            work::Action::Check => {
                lesson_presentation::prose(
                    ui,
                    "Choose Check example to verify the complete example.",
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
    let label = ui.strong("Saved IPv6 rules · simulated file");
    lesson_presentation::monospace(ui, core::FILE, true);
    lesson_presentation::prose(
        ui,
        "Each rule is one file line. Scroll sideways to read long rules.",
    );
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|fonts| fonts.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let mut text = s.editor.clone();
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 9. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, scrolled) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt("v6-startup-editor-scroll")
            .max_height(size.y)
            .min_scrolled_height(height)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt("v6-startup-file")
                        .layouter(&mut layouter)
                        .font(egui::TextStyle::Monospace)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        // Keep one firewall rule on one logical display line.
                        // The surrounding fixed viewport owns horizontal scrolling.
                        .desired_width(f32::INFINITY)
                        .desired_rows(9)
                        .char_limit(32768),
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
        let r = ui.label(if s.editor == s.world.saved_filter {
            "Saved · load separately"
        } else {
            "Unsaved changes"
        });
        m.record(ui, "file-state", &r);
    });
    controls::collapsing_header(ui,"Reload saved file · discards this editor draft").show(ui,|ui|{lesson_presentation::prose(ui,"This replaces only the editor with the saved file. It does not change current rules or erase the other workspace.");let r=ui.add(button("Reload saved file"));m.record(ui,"reload-editor",&r);if r.clicked(){apply(s,t,work::Action::ReloadEditor,n);}});
}
fn trials(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    ui.strong("Current traffic · supplied sample conversations");
    let mut at = s.selected_case;
    controls::form_row(ui, "Traffic case", |ui, w| {
        let r = controls::dropdown(ui, "v6-startup-case", w)
            .selected_text(&t.traffic.cases[at].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in t.traffic.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut at, i, &c.label);
                    m.record(ui, &format!("case:{i}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "case", &r.response);
    });
    if at != s.selected_case {
        apply(s, t, work::Action::SelectCase { case: at }, n);
    }
    let r = ui.add(button("Run traffic test"));
    m.record(ui, "run-traffic", &r);
    if r.clicked() {
        apply(s, t, work::Action::RunCase, n);
    }
    if let Some(trial) = &s.last_trial {
        let passed = trial.steps.iter().filter(|s| s.check.passed).count();
        let r = ui.strong(format!(
            "Actual current traffic · {passed}/{} events pass",
            trial.steps.len()
        ));
        m.record(ui, "traffic-result", &r);
        for event in &trial.steps {
            lesson_presentation::prose(
                ui,
                &format!(
                    "{} · {}\n{}",
                    if event.check.passed {
                        "Passed"
                    } else {
                        "Needs work"
                    },
                    event.check.label,
                    event.check.detail
                ),
            );
        }
    } else {
        lesson_presentation::prose(
            ui,
            "No separate traffic report for the current rules and selection.",
        );
    }
    ui.separator();
    ui.strong("Fresh startup · saved rules and provided loader");
    lesson_presentation::prose(
        ui,
        "Starts a separate simulated table with ACCEPT defaults and your saved loader settings. Your current table stays unchanged.",
    );
    let r = ui.add(button("Run startup test"));
    m.record(ui, "startup", &r);
    if r.clicked() {
        apply(s, t, work::Action::RunStartup, n);
    }
    if let Some(startup) = &s.last_startup {
        let pass = startup.checks.iter().filter(|c| c.passed).count();
        let r = ui.strong(format!(
            "Actual startup · {pass}/{} outcomes pass",
            startup.checks.len()
        ));
        m.record(ui, "startup-result", &r);
        controls::collapsing_header(ui, "Inspect actual startup results").show(ui, |ui| {
            for c in &startup.checks {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} · {}\n{}",
                        if c.passed { "Passed" } else { "Needs work" },
                        c.label,
                        c.detail
                    ),
                );
            }
        });
    } else {
        lesson_presentation::prose(
            ui,
            "No separate startup report for the current saved file and enablement.",
        );
    }
}
impl LabApp {
    pub(super) fn v6_startup_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let (
            engine::Goal::LinuxV6Startup { definition },
            engine::State::LinuxV6Startup { simulation },
        ) = (&source.goal, &original.state)
        else {
            return;
        };
        if let Err(e) = definition.bound(simulation) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let first = egui::Id::new(("v6-startup-first", &source.id, original.visited));
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
                    self.metrics.v6_startup.record(ui, key, &r);
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
                .v6_startup
                .record(ui, "introduction-content", &intro.objective);
            lesson_presentation::worked(
                ui,
                "Separate worked example · expected configuration, not actual run results.",
            );
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics.v6_startup.record(ui, "worked", &r.response);
            controls::collapsing_header(ui, "Complete worked sequence · example only")
                .show(ui, |ui| worked(ui, &source, definition));
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.v6_startup.record(ui, "try-example", &r);
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
                self.metrics.v6_startup.record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
                let r=linux_v6_page::topology(ui,&t.traffic);self.metrics.v6_startup.record(ui,"topology",&r);
                ui.horizontal_wrapped(|ui|{ui.label(format!("Provided loader: {}",match s.world.status{core::Status::Inactive=>"Inactive",core::Status::Loaded=>"Loaded (oneshot finished)",core::Status::Failed{..}=>"Failed"}));ui.label(if s.world.enabled{"Enabled at startup"}else{"Disabled at startup"});});
                if practice{lesson_presentation::practice_step(ui,&source,0);}
                if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*54.{ui.columns(2,|cols|{if practice{lesson_presentation::practice_step(&mut cols[0],&source,1);}console(&mut cols[0],t,s,&source,&mut self.metrics.v6_startup,&mut self.notice,(practice,&mut self.focus_input));if practice{lesson_presentation::practice_step(&mut cols[1],&source,2);}editor(&mut cols[1],t,s,&mut self.metrics.v6_startup,&mut self.notice);});}else{if practice{lesson_presentation::practice_step(ui,&source,1);}console(ui,t,s,&source,&mut self.metrics.v6_startup,&mut self.notice,(practice,&mut self.focus_input));if practice{lesson_presentation::practice_step(ui,&source,2);}editor(ui,t,s,&mut self.metrics.v6_startup,&mut self.notice);}
                trials(ui,t,s,&mut self.metrics.v6_startup,&mut self.notice);
                if practice{controls::collapsing_header(ui,"Try a persistence regression").show(ui,|ui|{lesson_presentation::practice_step(ui,&source,3);});}
                let r=ui.add(primary(ui,if practice{"Check example"}else{"Check work"}));self.metrics.v6_startup.record(ui,"check",&r);if r.clicked(){apply(s,t,work::Action::Check,&mut self.notice);}
                if s.checked{let checks=s.checks(t);let pass=checks.iter().filter(|c|c.passed).count();let r=ui.strong(if pass==checks.len(){if practice{"Example checks passed".into()}else{"Assignment complete".into()}}else{format!("Not complete · {pass}/{} outcomes pass",checks.len())});self.metrics.v6_startup.result=Some((pass,checks.len(),r.rect));self.metrics.v6_startup.record(ui,"result",&r);for c in checks.iter().filter(|c|!c.passed){let r=lesson_presentation::prose(ui,&format!("Needs work · {}\n{}",c.label,c.detail));self.metrics.v6_startup.record(ui,"failed-case",&r);}controls::collapsing_header(ui,"Inspect passing checks").show(ui,|ui|{for c in checks.iter().filter(|c|c.passed){lesson_presentation::prose(ui,&format!("Passed · {}\n{}",c.label,c.detail));}});if pass==checks.len(){lesson_presentation::prose(ui,if practice{"The example policy works in both the current table and the supplied startup trial. Apply the same reasoning independently to the assignment."}else{&source.explanation});}}
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
                        engine::State::LinuxV6Startup {
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
