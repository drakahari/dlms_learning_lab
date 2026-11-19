//! Supplied image support beside a bounded module terminal; no host operation.
use super::*;
use dlms_learning_lab::{
    guest_driver as core, guest_driver_source as binding, guest_driver_workspace as work,
    rpm_packages as rpm,
};

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
fn requirement(r: &rpm::Requirement) -> String {
    let tail = r
        .constraint
        .as_ref()
        .map(|c| {
            let relation = match c.relation {
                rpm::Relation::Eq => "=",
                rpm::Relation::Lt => "<",
                rpm::Relation::Le => "<=",
                rpm::Relation::Ge => ">=",
                rpm::Relation::Gt => ">",
            };
            format!(" {relation} {}", c.evr.as_str())
        })
        .unwrap_or_default();
    format!("{}{tail}", r.capability)
}
fn archive_choice(ui: &mut egui::Ui, t: &work::Task, s: &mut work::Scope, m: &mut Metrics) {
    controls::form_row(ui, "Supplied archive", |ui, width| {
        let label = t
            .facts
            .packages
            .archives
            .get(&s.selected_archive)
            .map(|a| format!("{} {}", a.identity.name, a.evr.as_str()))
            .unwrap_or_else(|| "Choose a supplied archive".into());
        let r = controls::dropdown(ui, "guest-package-archive", width)
            .selected_text(label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let start = m.controls.len();
                for id in &s.runtime.packages.available {
                    let a = &t.facts.packages.archives[id];
                    let r = ui.selectable_value(
                        &mut s.selected_archive,
                        id.clone(),
                        format!("{} {}", a.identity.name, a.evr.as_str()),
                    );
                    m.record(ui, &format!("archive:{id}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for row in &mut m.controls[start..] {
                    row.2 = clip;
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "archive", &r.response);
    });
}
fn packages(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    practice: bool,
    m: &mut Metrics,
) {
    ui.strong("Supplied package workspace");
    if practice {
        lesson_presentation::practice_step(ui, source, 0);
    }
    archive_choice(ui, t, s, m);
    if let Some(a) = t.facts.packages.archives.get(&s.selected_archive) {
        let installed = s
            .runtime
            .packages
            .installed
            .iter()
            .find(|p| p.identity == a.identity)
            .map(|p| t.facts.packages.archives[&p.archive].evr.as_str());
        lesson_presentation::prose(
            ui,
            &format!(
                "{} · x86_64 · installed version: {}",
                a.identity.name,
                installed.unwrap_or("not installed")
            ),
        );
        if !a.requires.is_empty() {
            lesson_presentation::prose(
                ui,
                &format!(
                    "Requires: {}",
                    a.requires
                        .iter()
                        .map(requirement)
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            );
        }
        if !a.provides.is_empty() {
            lesson_presentation::prose(
                ui,
                &format!(
                    "Provides: {}",
                    a.provides
                        .iter()
                        .map(|(n, v)| format!(
                            "{n}{}",
                            v.as_ref()
                                .map(|e| format!(" = {}", e.as_str()))
                                .unwrap_or_default()
                        ))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
            );
        }
        ui.label("Recorded object files in this archive");
        controls::nested_scroll(
            ui,
            egui::ScrollArea::horizontal()
                .id_salt("guest-archive-files")
                .auto_shrink([false, true]),
            |ui| {
                for path in a.files.keys() {
                    ui.monospace(path);
                }
            },
        );
    }
    controls::form_row(ui, "Package action", |ui, width| {
        let r = controls::dropdown(ui, "guest-package-operation", width)
            .selected_text(
                s.operation
                    .map(work::PackageOperation::label)
                    .unwrap_or("Choose an action"),
            )
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                let start = m.controls.len();
                for op in [
                    work::PackageOperation::Install,
                    work::PackageOperation::Replace,
                    work::PackageOperation::Reinstall,
                    work::PackageOperation::Remove,
                ] {
                    let r = ui.selectable_value(&mut s.operation, Some(op), op.label());
                    m.record(ui, &format!("operation:{op:?}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for row in &mut m.controls[start..] {
                    row.2 = clip;
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "operation", &r.response);
    });
    if s.operation == Some(work::PackageOperation::Remove) {
        lesson_presentation::prose(
            ui,
            "Removal affects the installed package with this name, including its owned files; selecting a different archive version does not select an older installed copy.",
        );
    }
    let r = ui.add_enabled(
        !s.selected_archive.is_empty() && s.operation.is_some(),
        button("Apply package change"),
    );
    m.record(ui, "apply-package", &r);
    if r.clicked() {
        let result = s.selected_action(t).and_then(|a| s.apply(t, &a));
        if let Err(e) = result {
            s.note_package_failure(&e);
        }
    }
    controls::collapsing_header(ui, "Inspect image objects and lookup").show(ui, |ui| {
        let present = s.runtime.present(&t.facts).unwrap_or_default();
        for (name, module) in &t.facts.modules.modules {
            lesson_presentation::prose(
                ui,
                &format!(
                    "{name} · file: {} · lookup: {} · driver: {}",
                    if present.contains(name) {
                        "Present"
                    } else {
                        "Missing"
                    },
                    if s.runtime.modules.index.contains(name) {
                        "Indexed"
                    } else {
                        "Absent"
                    },
                    if s.runtime.modules.loaded.contains_key(name) {
                        "Loaded"
                    } else {
                        "Not loaded"
                    }
                ),
            );
            controls::nested_scroll(
                ui,
                egui::ScrollArea::horizontal()
                    .id_salt(("guest-object-path", name))
                    .auto_shrink([false, true]),
                |ui| {
                    ui.monospace(&module.path);
                },
            );
        }
    });
}
fn terminal(
    ui: &mut egui::Ui,
    t: &work::Task,
    s: &mut work::Scope,
    source: &Scenario,
    phase: (bool, u64),
    m: &mut Metrics,
    focus: &mut bool,
) {
    let (practice, visited) = phase;
    ui.strong("Module command practice");
    if practice {
        lesson_presentation::practice_step(ui, source, 1);
    }
    if practice {
        let commands: Vec<_> = source
            .guide
            .iter()
            .flat_map(|g| g.commands.iter())
            .filter(|c| !c.simulator_only)
            .collect();
        if !commands.is_empty() {
            let key = egui::Id::new(("guest-example-command-choice", &source.id, visited));
            let mut selected = ui
                .ctx()
                .data(|d| d.get_temp::<usize>(key))
                .unwrap_or(0)
                .min(commands.len() - 1);
            controls::form_row(ui, "Example command", |ui, width| {
                let r = controls::dropdown(ui, "guest-example-command", width)
                    .selected_text(&commands[selected].command)
                    .show_ui(ui, |ui| {
                        controls::fit_popup_contents(ui);
                        let start = m.controls.len();
                        for (n, c) in commands.iter().enumerate() {
                            let r = ui.selectable_value(&mut selected, n, &c.command);
                            m.record(ui, &format!("example-command:{n}"), &r);
                        }
                        let clip = controls::popup_content_clip(ui);
                        for row in &mut m.controls[start..] {
                            row.2 = clip;
                        }
                    });
                controls::retain_dropdown_focus(ui, &r.response);
                m.record(ui, "example-command", &r.response);
            });
            lesson_presentation::prose(ui, &commands[selected].explanation);
            let r = ui.add(button("Insert command"));
            m.record(ui, "insert-command", &r);
            if r.clicked() {
                s.input = commands[selected].command.clone();
                *focus = true;
            }
            ui.ctx().data_mut(|d| d.insert_temp(key, selected));
        }
    }
    let height = (ui.text_style_height(&egui::TextStyle::Monospace) * 7.).clamp(140., 300.);
    egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui| {
        ui.label("Actual command and package results");
        controls::nested_scroll(ui,egui::ScrollArea::both().id_salt("guest-module-output").auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui| {
            if s.output.is_empty() {lesson_presentation::prose(ui,"Type a supported command below, then press Enter or Run. Package actions also report their results here.");}
            else{ui.add(egui::Label::new(RichText::new(&s.output).monospace()).extend());}
        });
    });
    let mut run = false;
    let run_width = controls::button_width(ui, "Run");
    let row = ui.available_width()
        > run_width
            + ui.spacing().item_spacing.x
            + ui.text_style_height(&egui::TextStyle::Body) * 8.;
    let mut input = |ui: &mut egui::Ui, run: &mut bool, m: &mut Metrics| {
        let edit = shared_libraries_page::fitting_input(
            ui,
            &s.input,
            egui::Id::new(("guest-module-command", &source.id, visited, practice)),
            4096,
            ui.is_enabled(),
            "Command · Enter to run",
        );
        m.fields
            .push(("command".into(), edit.response.rect, edit.response.id));
        m.record(ui, "command", &edit.response);
        if let Some(text) = edit.value {
            s.input = text;
        }
        if *focus && ui.is_enabled() {
            edit.response.request_focus();
            *focus = false;
        }
        *run |= ui.is_enabled()
            && (edit.response.has_focus() || edit.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if edit.rejected {
            lesson_presentation::prose(
                ui,
                "Use one command of at most 4,096 bytes; the existing draft was retained.",
            );
        }
    };
    if row {
        ui.horizontal(|ui| {
            let width = (ui.available_width() - run_width - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(width, controls::height(ui)),
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
        match s.apply(
            t,
            &work::Action::Run {
                input: command.clone(),
            },
        ) {
            Ok(_) => s.input.clear(),
            Err(e) => s.note_failure(&command, &e),
        }
        *focus = true;
    }
    controls::collapsing_header(ui, "Supported module commands").show(ui, |ui| {
        lesson_presentation::prose(ui, core::HELP);
    });
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
            "Unchecked · Check configuration tests both the current drivers and separate unloaded requests from the stored image.",
        );
        m.record(ui, "unchecked", &r);
        return;
    }
    let rows = t.checks(&s.runtime);
    let passed = rows.iter().filter(|r| r.passed).count();
    let r = ui.strong(if passed == rows.len() {
        if practice {
            "Example checks pass"
        } else {
            "Assignment complete"
        }
    } else {
        "Configuration needs attention"
    });
    m.record(ui, "result-title", &r);
    m.result = Some((rows.len(), rows.len() - passed, r.rect));
    lesson_presentation::prose(
        ui,
        &format!("{passed}/{} required outcomes pass.", rows.len()),
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
    if passed == rows.len() {
        lesson_presentation::prose(ui, &source.explanation);
    }
}
impl LabApp {
    pub(super) fn guest_driver_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let (engine::Goal::GuestDrivers { definition }, engine::State::GuestDrivers { simulation }) =
            (&source.goal, &original.state)
        else {
            return;
        };
        let visited = original.visited;
        let first = egui::Id::new(("guest-first-workspace-frame", &source.id, visited));
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
                    self.metrics.guest_driver.record(ui, key, &r);
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
                "Separate example image · read the method, then try its package and driver controls.",
            );
            let r = ui.scope(|ui| lesson_presentation::worked_rows(ui, &source));
            self.metrics.guest_driver.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try this example"));
            self.metrics.guest_driver.record(ui, "try", &r);
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
            lesson_presentation::prose(
                ui,
                &format!(
                    "Represented x86_64 guest · running kernel {}",
                    t.facts.modules.kernel_release
                ),
            );
            for device in &t.facts.modules.devices {
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} ({}) · driver in use: {}",
                        device.description,
                        device.slot,
                        if s.runtime.modules.loaded.contains_key(&device.driver) {
                            &device.driver
                        } else {
                            "none"
                        }
                    ),
                );
            }
            ui.separator();
            ui.push_id(("guest-image-work", &source.id, practice), |ui| {
                ui.add_enabled_ui(active, |ui| {
                    if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 45. {
                        ui.columns(2, |columns| {
                            packages(
                                &mut columns[0],
                                t,
                                s,
                                &source,
                                practice,
                                &mut self.metrics.guest_driver,
                            );
                            terminal(
                                &mut columns[1],
                                t,
                                s,
                                &source,
                                (practice, visited),
                                &mut self.metrics.guest_driver,
                                &mut self.focus_input,
                            );
                        });
                    } else {
                        packages(ui, t, s, &source, practice, &mut self.metrics.guest_driver);
                        ui.add_space(controls::GAP);
                        terminal(
                            ui,
                            t,
                            s,
                            &source,
                            (practice, visited),
                            &mut self.metrics.guest_driver,
                            &mut self.focus_input,
                        );
                    }
                    if practice {
                        lesson_presentation::practice_step(ui, &source, 2);
                    }
                    ui.separator();
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(primary(ui, "Check configuration"));
                        self.metrics.check = Some(r.rect);
                        self.metrics.guest_driver.record(ui, "check", &r);
                        if r.clicked()
                            && let Err(e) = s.apply(t, &work::Action::Check)
                        {
                            self.notice = Some(e);
                        }
                        if practice {
                            let r = ui.add(button("Reset example only"));
                            self.metrics.guest_driver.record(ui, "reset-example", &r);
                            if r.clicked() {
                                match work::Session::new(definition) {
                                    Ok(fresh) => *s = fresh.practice,
                                    Err(e) => self.notice = Some(e),
                                }
                            }
                        }
                    });
                    results(ui, t, s, &source, practice, &mut self.metrics.guest_driver);
                });
            });
            controls::collapsing_header(ui, "Optional help and simulation scope").show(ui, |ui| {
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
                    controls::collapsing_header(ui, "Worked approach · reveals the assignment")
                        .show(ui, |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                            for (n, action) in definition.reference.iter().enumerate() {
                                match action {
                                    work::Action::Run { input } => {
                                        controls::nested_scroll(
                                            ui,
                                            egui::ScrollArea::horizontal()
                                                .id_salt(("guest-worked-command", n))
                                                .auto_shrink([false, true]),
                                            |ui| {
                                                lesson_presentation::monospace(ui, input, false);
                                            },
                                        );
                                    }
                                    work::Action::Package { action } => {
                                        lesson_presentation::prose(
                                            ui,
                                            &format!(
                                                "Native package workspace: {}",
                                                package_instruction(&definition.assignment, action)
                                            ),
                                        );
                                    }
                                    work::Action::Check => {
                                        lesson_presentation::prose(
                                            ui,
                                            "Select Check configuration to verify the full task.",
                                        );
                                    }
                                }
                            }
                        });
                }
                lesson_presentation::prose(ui, &source.simplifications);
                for reference in &source.sources {
                    ui.hyperlink_to(reference, reference);
                }
            });
        }
        if active && next != before {
            match definition.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::GuestDrivers {
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
fn package_instruction(t: &work::Task, a: &rpm::Action) -> String {
    let label = |id: &str| {
        t.facts
            .packages
            .archives
            .get(id)
            .map(|a| format!("{} {}", a.identity.name, a.evr.as_str()))
            .unwrap_or_else(|| id.into())
    };
    match a {
        rpm::Action::InstallSet { archives } => format!(
            "choose {}; Install, then select Apply package change",
            archives
                .iter()
                .map(|id| label(id))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        rpm::Action::UpgradeSet { archives } => format!(
            "choose {}; Replace / upgrade, then select Apply package change",
            archives
                .iter()
                .map(|id| label(id))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        rpm::Action::ReinstallExact { archive } => {
            format!(
                "choose {}; Reinstall, then select Apply package change",
                label(archive)
            )
        }
        rpm::Action::EraseSet { packages } => format!(
            "choose the supplied archive for {}; Remove installed package, then select Apply package change",
            packages
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}
