//! Native text editing and a finite, hypothetical boot trace. No command execution.
use super::*;
use dlms_learning_lab::{boot_entry as core, boot_entry_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub editor: Option<(egui::Rect, egui::Id)>,
    /// Laid-out text, excluding the editor's intentionally empty rows.
    pub editor_text: Option<egui::Rect>,
    pub generated_start: Option<egui::Rect>,
    pub result: Option<bool>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn split_width(ui: &egui::Ui, t: &work::Task) -> f32 {
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let line_width = t
        .starter
        .lines()
        .chain(t.reference.lines())
        .map(|line| ui.fonts_mut(|f| f.layout_no_wrap(line.into(), font.clone(), color).size().x))
        .fold(0_f32, f32::max);
    // Both columns need real content width. At enlarged text sizes a fixed
    // breakpoint can leave a UUID or kernel argument hidden in the editor.
    (ui.text_style_height(&egui::TextStyle::Body) * 48.)
        .max(2. * (line_width + 2. * f32::from(controls::PAD_X)) + ui.spacing().item_spacing.x)
}
fn facts(ui: &mut egui::Ui, t: &work::Task) {
    if let Some(policy) = &t.generator {
        ui.strong("Supplied menu templates");
        lesson_presentation::prose(
            ui,
            "These fixed templates provide image paths, system UUID and normal/recovery entries. Generation adds your settings; it does not discover files or run scripts.",
        );
        if let Ok(menu) = dlms_learning_lab::grub_menu::Menu::parse(&policy.templates) {
            for item in menu.items {
                lesson_presentation::monospace(
                    ui,
                    &format!(
                        "{} · {} · {}",
                        item.id,
                        if policy.normal_ids.contains(&item.id) {
                            "normal"
                        } else {
                            "recovery"
                        },
                        item.entry.linux
                    ),
                    true,
                );
            }
        }
        controls::collapsing_header(ui, "Read supplied templates · optional").show(ui, |ui| {
            lesson_presentation::code_value(
                ui,
                "generation-templates",
                &policy.templates,
                ui.available_width(),
            );
        });
        ui.add_space(controls::GAP);
    }
    ui.strong("Supplied boot partition and filesystems");
    lesson_presentation::prose(
        ui,
        "Paths below start at virtual $BOOT. A linux or initrd path names one of these supplied images; it is not a path on your computer.",
    );
    ui.strong("Kernel images");
    for k in &t.inventory.kernels {
        lesson_presentation::monospace(ui, &k.path, true);
        lesson_presentation::prose(
            ui,
            &format!(
                "Release {} · built-in drivers: {}",
                k.release,
                if k.builtin_drivers.is_empty() {
                    "none".into()
                } else {
                    k.builtin_drivers.join(", ")
                }
            ),
        );
    }
    ui.add_space(controls::GAP);
    ui.strong("Initramfs images");
    for i in &t.inventory.initramfs {
        lesson_presentation::monospace(ui, &i.path, true);
        lesson_presentation::prose(
            ui,
            &format!(
                "For {} · drivers: {} · UUID lookup {}",
                i.release,
                if i.drivers.is_empty() {
                    "none".into()
                } else {
                    i.drivers.join(", ")
                },
                if i.uuid_lookup {
                    "available"
                } else {
                    "unavailable"
                }
            ),
        );
    }
    for r in &t.inventory.roots {
        ui.add_space(controls::GAP);
        ui.strong(&r.label);
        lesson_presentation::monospace(ui, &format!("UUID={}", r.uuid), true);
        lesson_presentation::prose(
            ui,
            &format!(
                "Needs {} · init program {}",
                r.required_drivers.join(" + "),
                if r.init_present {
                    "available"
                } else {
                    "absent"
                }
            ),
        );
    }
}
struct EditorControls<'a> {
    metrics: &'a mut Metrics,
    focus: &'a mut bool,
    notice: &'a mut Option<String>,
}
fn editor(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    t: &work::Task,
    lesson: (&Scenario, bool),
    controls: &mut EditorControls<'_>,
) {
    let (s, practice) = lesson;
    let EditorControls {
        metrics: m,
        focus,
        notice,
    } = controls;

    if practice {
        lesson_presentation::practice_step(ui, s, 0);
    }
    let label = ui.strong(if t.generator.is_some() {
        "Persistent boot settings"
    } else if t.menu.is_some() {
        "GRUB menu editor"
    } else {
        "Boot entry editor"
    });
    lesson_presentation::monospace(ui, &t.entry_path, true);
    let width = ui.available_width();
    let rows = if t.generator.is_some() {
        6
    } else if t.menu.is_some() {
        15
    } else {
        9
    };
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let row_height = if t.menu.is_some() || t.generator.is_some() {
        ui.fonts_mut(|f| f.row_height(&font))
    } else {
        ui.text_style_height(&egui::TextStyle::Monospace)
    };
    let height = row_height * rows as f32 + 2. * f32::from(controls::PAD_Y);
    let mut text = draft.source.clone();
    let color = palette(ui).text;
    let mut layouter = move |ui: &egui::Ui, text: &dyn egui::TextBuffer, _: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(text.as_str().into(), font.clone(), color))
    };
    let (viewport, response) = controls::fixed_viewport(ui, egui::vec2(width, height), |ui| {
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("boot-entry-document")
                .max_width(width)
                .min_scrolled_width(width)
                .max_height(height)
                .auto_shrink([false, false]),
            |ui| {
                let output = egui::TextEdit::multiline(&mut text)
                    .code_editor()
                    .lock_focus(false)
                    .layouter(&mut layouter)
                    .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                    .desired_rows(rows)
                    .desired_width(
                        (ui.available_width() - 2. * f32::from(controls::PAD_X)).max(24.),
                    )
                    .id(egui::Id::new(("boot-entry-editor", &t.id)))
                    .char_limit(core::MAX_SOURCE)
                    .show(ui);
                m.editor_text = Some(egui::Rect::from_min_size(
                    output.galley_pos,
                    output.galley.size(),
                ));
                output.response.labelled_by(label.id)
            },
        )
        .inner
    });
    m.editor = Some((viewport, response.id));
    m.controls.push(("editor".into(), viewport, ui.clip_rect()));
    if **focus && ui.is_enabled() {
        response.request_focus();
        **focus = false;
    }
    if text != draft.source {
        **notice = draft.edit(text).err();
    }
    if let Some(error) = notice.as_deref() {
        ui.strong("Input needs attention");
        lesson_presentation::prose(ui, error);
    }
    if practice {
        lesson_presentation::practice_step(ui, s, 1);
    }
    if t.generator.is_some() {
        let r = ui.add_sized(
            egui::vec2(ui.available_width(), controls::height(ui)),
            button("Generate menu · simulation"),
        );
        record(m, "generate", &r, ui);
        if r.clicked() {
            **notice = draft.generate(t).err();
        }
        if let Some(g) = &draft.generated {
            lesson_presentation::prose(
                ui,
                if g.settings == draft.source {
                    "Generated menu: current settings."
                } else {
                    "Generated menu: OUTDATED. The edited settings have not been generated."
                },
            );
            lesson_presentation::prose(
                ui,
                &format!(
                    "Generated timeout: {}",
                    match g.timeout {
                        -1 => "wait for a selection".into(),
                        0 => "start immediately".into(),
                        n => format!("{n} seconds"),
                    }
                ),
            );
            let generated = controls::collapsing_header(ui, "Inspect generated menu · read only")
                .show(ui, |ui| {
                    lesson_presentation::prose(ui, "Long lines wrap to fit this view.");
                    let text = lesson_presentation::monospace(
                        ui,
                        &format!("set timeout={}\n{}", g.timeout, g.menu),
                        true,
                    );
                    record(m, "generated-text", &text, ui);
                    let font = egui::TextStyle::Monospace.resolve(ui.style());
                    let rows = ui.fonts_mut(|f| f.row_height(&font)) * 9.;
                    m.generated_start = Some(egui::Rect::from_min_size(
                        text.rect.min,
                        egui::vec2(text.rect.width(), rows.min(text.rect.height())),
                    ));
                });
            record(m, "generated-output", &generated.header_response, ui);
        } else {
            lesson_presentation::prose(
                ui,
                "Generated menu: none yet. Editing the settings alone does not create one.",
            );
        }
        if practice {
            lesson_presentation::practice_step(ui, s, 2);
        }
    }
    let response = ui.add_sized(
        egui::vec2(ui.available_width(), controls::height(ui)),
        button(if t.generator.is_some() {
            "Test generated menu · simulation"
        } else if t.menu.is_some() {
            "Test menu · simulation"
        } else {
            "Test entry · simulation"
        }),
    );
    record(m, "test", &response, ui);
    if response.clicked() {
        **notice = draft.check(t).err();
    }
}
fn results(
    ui: &mut egui::Ui,
    draft: &work::Draft,
    t: &work::Task,
    practice: bool,
    m: &mut Metrics,
) {
    if draft.checked.is_none() {
        lesson_presentation::prose(ui, "Actual result: not tested for the current text.");
        return;
    }
    let checks = draft.checks(t);
    let passed = checks.iter().all(|c| c.0);
    m.result = Some(passed);
    let header = lesson_presentation::lead(
        ui,
        "ACTUAL TEST RESULT",
        if passed {
            if practice {
                "Practice outcomes match"
            } else {
                "Assignment complete"
            }
        } else {
            "Repair needed"
        },
    );
    record(m, "result", &header, ui);
    match draft.report(t) {
        Err(e) => {
            let readiness = t.generator.is_some()
                && draft
                    .generated
                    .as_ref()
                    .is_none_or(|g| g.settings != draft.source);
            let category = if readiness {
                "Not ready to test".into()
            } else {
                format!("{:?}", e.kind)
            };
            lesson_presentation::prose(ui, &format!("{category}: {}", e.message));
        }
        Ok(r) => {
            // A good selected path does not imply a good recovery configuration.
            // Put failed generated-menu requirements beside the action before
            // showing the detailed successful handoff; retain legacy layout.
            if t.generator.is_some() {
                for (_, message) in checks.iter().skip(r.stages.len() + 1).filter(|c| !c.0) {
                    let response = ui.strong("Needs attention");
                    record(m, "failed-requirement", &response, ui);
                    lesson_presentation::prose(ui, message);
                }
            }
            for step in &r.stages {
                let response = ui.strong(format!(
                    "{} · {}",
                    if step.passed { "Reached" } else { "Blocked" },
                    step.title
                ));
                if t.generator.is_some() {
                    record(m, "boot-stage", &response, ui);
                }
                lesson_presentation::prose(ui, &step.detail);
            }
            if r.passed() && !passed && t.generator.is_none() {
                lesson_presentation::prose(
                    ui,
                    if t.menu.is_some() || t.generator.is_some() {
                        "This selected entry reaches a modeled handoff, but one or more requirements below still need attention."
                    } else {
                        "This entry can reach a modeled handoff, but it does not yet meet this assignment's requested system, kernel, mode or target."
                    },
                );
            }
            for (ok, message) in checks.iter().skip(r.stages.len() + 1) {
                if t.generator.is_some() && !ok {
                    continue;
                }
                lesson_presentation::prose(
                    ui,
                    &format!(
                        "{} · {message}",
                        if *ok {
                            "Matches task"
                        } else {
                            "Needs attention"
                        }
                    ),
                );
            }
        }
    }
}
impl LabApp {
    pub(super) fn boot_entry_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let admitted = work::source(&source)
            .and_then(|d| work::context(&self.progress.attempts[&source.id], &source).map(|_| d));
        let definition = match admitted {
            Ok(d) => d,
            Err(e) => {
                self.notice = Some(e);
                return;
            }
        };
        let engine::State::BootEntry { simulation } = &self.progress.attempts[&source.id].state
        else {
            return;
        };
        let mut next = (**simulation).clone();
        let before = next.clone();
        let active = ui.is_enabled()
            && !(self.reset_pending
                || self.settings_open
                || self.theme_preview
                || self.data_tools.modal_open());
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        self.metrics.boot_entry = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    record(&mut self.metrics.boot_entry, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        next.view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                        self.notice = None;
                    }
                }
            });
        }
        ui.push_id((&source.id,view as u8,challenge),|ui|{
   if view==teaching::View::Introduction{
    lesson_presentation::concepts(ui,&source,"Start with ordinary file paths, filesystem UUIDs and the difference between a current system and saved startup configuration. Those ideas are recapped below.",std::iter::empty());
    lesson_presentation::worked(ui,&definition.example.instructions);if let Some(content)=lesson_presentation::entry(&source){for (n,row) in content.worked.iter().enumerate(){ui.strong(&row.label);lesson_presentation::code_value(ui,("boot-worked",n),&row.value,ui.available_width());lesson_presentation::prose(ui,&row.explanation);ui.add_space(controls::GAP);}}
    ui.horizontal_wrapped(|ui|{for(v,key,label)in[(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));record(&mut self.metrics.boot_entry,key,&r,ui);if r.clicked(){next.view=v;self.lesson_scroll_reset=true;self.focus_input=true;self.notice=None;}}});
   }else{
    let practice=view==teaching::View::Example;let task=if practice{&definition.example}else{&definition.initial};let draft=if practice{&mut next.practice}else{&mut next.assignment};
    if practice{lesson_presentation::lead(ui,"PRACTICE TASK",&task.instructions);}else{lesson_presentation::assignment(ui,&source);}
    ui.add_enabled_ui(active,|ui|{
     self.metrics.boot_entry.side_by_side=ui.available_width()>=split_width(ui,task);
     if self.metrics.boot_entry.side_by_side{ui.columns(2,|cols|{editor(&mut cols[0],draft,task,(&source,practice),&mut EditorControls{metrics:&mut self.metrics.boot_entry,focus:&mut self.focus_input,notice:&mut self.notice});results(&mut cols[0],draft,task,practice,&mut self.metrics.boot_entry);facts(&mut cols[1],task);});}else{editor(ui,draft,task,(&source,practice),&mut EditorControls{metrics:&mut self.metrics.boot_entry,focus:&mut self.focus_input,notice:&mut self.notice});results(ui,draft,task,practice,&mut self.metrics.boot_entry);ui.add_space(controls::GAP);facts(ui,task);}
     if practice{lesson_presentation::practice_step(ui,&source,if task.generator.is_some(){3}else{2});}
     controls::collapsing_header(ui,"Hints and supported behavior · optional").show(ui,|ui|{for h in &source.hints{lesson_presentation::prose(ui,h);}lesson_presentation::prose(ui,if task.generator.is_some(){dlms_learning_lab::grub_settings::HELP}else if task.menu.is_some(){dlms_learning_lab::grub_menu::HELP}else{core::HELP});});
     if practice{let worked=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{lesson_presentation::code_value(ui,"boot-entry-worked",&task.reference,ui.available_width());let r=ui.add(button("Fill worked example"));record(&mut self.metrics.boot_entry,"fill",&r,ui);if r.clicked(){self.notice=draft.edit(task.reference.clone()).err();}lesson_presentation::prose(ui,if task.generator.is_some(){"Fills this practice settings draft. Generate menu, then Test generated menu; filling text does not generate, test or complete the assignment."}else if task.menu.is_some(){"Fills only this practice menu. Test menu checks the selected handoff; filling text does not test it."}else{"Fills only this practice editor. Test entry runs the modeled handoff; filling text does not check it."});});if task.menu.is_some()||task.generator.is_some(){record(&mut self.metrics.boot_entry,"worked",&worked.header_response,ui);}}
    });
   }
  });
        if let Some((r, id)) = self.metrics.boot_entry.editor {
            self.metrics.focus = Some(id);
            self.metrics.editor_viewport = Some(r);
        }
        self.metrics.check = self
            .metrics
            .boot_entry
            .controls
            .iter()
            .find(|r| r.0 == "test")
            .map(|r| r.1);
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::BootEntry {
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
