//! Supplied-machine EFI recovery console; every command is interpreted in process.
use super::*;
use dlms_learning_lab::{efi_loader as core, efi_loader_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub input: Option<(egui::Rect, egui::Id)>,
    pub result: Option<bool>,
    pub side_by_side: bool,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn facts(ui: &mut egui::Ui, task: &work::Task, draft: &work::Draft) {
    ui.strong("Supplied machine · UEFI / GPT");
    lesson_presentation::prose(
        ui,
        "Each EFI System Partition contains FAT32 files. A Linux mount path tells the installer where to write. Firmware uses the partition identity and a path inside it.",
    );
    if let Some(media) = &task.media {
        lesson_presentation::prose(
            ui,
            &format!(
                "Recovery test: select the removable device with {} unavailable.",
                media.unavailable_disk
            ),
        );
    }
    for esp in &task.facts.esps {
        let device = format!("{}{}", esp.disk, esp.number);
        let role = if esp.removable {
            "removable ESP"
        } else {
            "fixed ESP"
        };
        lesson_presentation::monospace(ui, &format!("{device} · {role}"), true);
        if let Some(mount) = &esp.mount {
            lesson_presentation::monospace(ui, &format!("Mounted at {}", mount.path), true);
        } else {
            lesson_presentation::prose(ui, "No Linux mount supplied for this partition.");
        }
        let mut files = draft
            .current
            .files
            .iter()
            .filter(|file| file.partition_guid == esp.partition_guid)
            .peekable();
        if files.peek().is_none() {
            lesson_presentation::prose(ui, "No supplied EFI files here yet.");
        }
        for file in files {
            lesson_presentation::monospace(ui, file.path.as_str(), true);
        }
    }
    ui.add_space(controls::GAP);
    ui.strong("Boot directories");
    for directory in &task.facts.boot_directories {
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} → {}{}",
                directory.path, directory.disk, directory.partition
            ),
            true,
        );
        lesson_presentation::prose(
            ui,
            if directory.configuration.is_some() {
                "Contains supplied grub/grub.cfg."
            } else {
                "No supplied GRUB menu."
            },
        );
    }
    ui.add_space(controls::GAP);
    ui.strong("Current firmware order");
    let order = draft
        .current
        .boot_order
        .iter()
        .map(|number| format!("{number:04X}"))
        .collect::<Vec<_>>()
        .join(",");
    lesson_presentation::monospace(ui, &format!("BootOrder: {order}"), true);
}

fn practice_index(draft: &work::Draft, task: &work::Task) -> usize {
    if let Some(media) = &task.media {
        if draft.checked.is_none() {
            return 0;
        }
        return match draft.current.trace_removable(
            &task.facts,
            &task.required_esp,
            Some(&media.unavailable_disk),
        ) {
            Ok(trace) if trace.passed() => 3,
            Ok(trace) if trace.handoff.is_some() => 2,
            _ => 1,
        };
    }
    if draft.transcript.is_empty() {
        return 0;
    }
    let installed = draft
        .current
        .files
        .iter()
        .filter(|file| {
            file.partition_guid == task.required_esp
                && matches!(file.kind, core::FileKind::Grub { .. })
        })
        .collect::<Vec<_>>();
    if installed.is_empty() {
        return 1;
    }
    let registered = draft.current.options.iter().any(|entry| {
        entry.active
            && entry.partition_guid == task.required_esp
            && installed.iter().any(|file| file.path == entry.loader)
    });
    if registered && draft.checked.is_some() {
        3
    } else {
        2
    }
}

struct ConsoleControls<'a> {
    metrics: &'a mut Metrics,
    focus: &'a mut bool,
    notice: &'a mut Option<String>,
}
fn console(
    ui: &mut egui::Ui,
    draft: &mut work::Draft,
    t: &work::Task,
    lesson: (&Scenario, bool),
    controls: &mut ConsoleControls<'_>,
) {
    let (s, practice) = lesson;
    let ConsoleControls {
        metrics: m,
        focus,
        notice,
    } = controls;
    if practice {
        lesson_presentation::practice_step(ui, s, practice_index(draft, t));
    }
    let label = ui.strong("Simulated rescue console");
    let mut command = draft.command.clone();
    let r = ui
        .add(
            controls::singleline_fitting(ui, &mut command, ui.available_width())
                .id(egui::Id::new(("bios-command", &t.id)))
                .char_limit(1024)
                .hint_text("One supported command"),
        )
        .labelled_by(label.id);
    m.input = Some((r.rect, r.id));
    record(m, "input", &r, ui);
    if **focus && ui.is_enabled() {
        r.request_focus();
        **focus = false;
    }
    draft.command = command;
    // Typing does not mutate the machine. Only Run applies a command; a new
    // installation clears the tested observation in the core state model.
    ui.horizontal_wrapped(|ui| {
        let run = ui.add(button("Run command"));
        record(m, "run", &run, ui);
        if run.clicked() {
            **notice = draft.run(t).err();
            **focus = true;
        }
        let label = if t.media.is_some() {
            "Test recovery paths · simulation"
        } else {
            "Test boot chain · simulation"
        };
        let test = ui.add(button(label));
        record(m, "test", &test, ui);
        if test.clicked() {
            **notice = draft.test(t).err();
        }
    });
    if let Some(error) = notice.as_deref() {
        ui.strong("Input needs attention");
        lesson_presentation::prose(ui, error);
    }
    if draft.checked.is_some() {
        let rows = t.checks(draft);
        let passed = rows.iter().all(|r| r.0);
        m.result = Some(passed);
        let header = ui.strong(if passed {
            if practice {
                "Practice outcomes match"
            } else {
                "Assignment complete"
            }
        } else {
            "Repair needed"
        });
        record(m, "result", &header, ui);
        if let Some(media) = &t.media {
            let normal = draft.current.trace(&t.facts).is_ok_and(|trace| {
                trace.started == Some(t.archive_entry)
                    && trace.handoff.as_ref().is_some_and(|report| report.passed())
            });
            let recovery = draft
                .current
                .trace_removable(&t.facts, &t.required_esp, Some(&media.unavailable_disk))
                .is_ok_and(|trace| trace.passed());
            lesson_presentation::prose(
                ui,
                if normal {
                    "Normal startup: reached Linux."
                } else {
                    "Normal startup: repair needed."
                },
            );
            lesson_presentation::prose(
                ui,
                if recovery {
                    "Removable recovery: reached Linux."
                } else {
                    "Removable recovery: repair needed."
                },
            );
        }

        // Failed requirements precede successful stages, including preservation:
        // a good handoff must not hide damage to another loader.
        for (_, message) in rows.iter().filter(|r| !r.0) {
            lesson_presentation::prose(ui, &format!("Needs attention · {message}"));
        }
        if passed && t.media.is_none() {
            lesson_presentation::prose(
                ui,
                "The intended EFI entry reaches Linux. The archive entry and its files remain available.",
            );
        }
        controls::collapsing_header(ui, "Boot attempts and verified results · optional").show(
            ui,
            |ui| {
                if let Ok(trace) = draft.current.trace(&t.facts) {
                    for attempt in trace.attempts {
                        lesson_presentation::prose(
                            ui,
                            &format!("Boot{:04X} · {}", attempt.number, attempt.outcome),
                        );
                    }
                }
                for (_, message) in rows.iter().filter(|r| r.0) {
                    lesson_presentation::prose(ui, &format!("Verified · {message}"));
                }
            },
        );
    } else {
        lesson_presentation::prose(
            ui,
            if t.media.is_some() {
                "Test both startup paths after making a change."
            } else {
                "Test the current installation and BootOrder after making a change."
            },
        );
    }
    ui.strong("Actual console output");
    let height = ui.text_style_height(&egui::TextStyle::Monospace) * 10.;
    controls::nested_scroll(
        ui,
        egui::ScrollArea::both()
            .id_salt(("bios-output", &t.id))
            .max_height(height)
            .min_scrolled_height(height)
            .stick_to_bottom(true)
            .auto_shrink([false, false]),
        |ui| {
            lesson_presentation::monospace(
                ui,
                if draft.transcript.is_empty() {
                    "No commands run yet."
                } else {
                    &draft.transcript
                },
                false,
            );
        },
    );
    ui.add_space(controls::GAP);
}

impl LabApp {
    pub(super) fn efi_loader_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let definition = match work::source(&source)
            .and_then(|d| work::context(&self.progress.attempts[&source.id], &source).map(|_| d))
        {
            Ok(d) => d,
            Err(e) => {
                self.notice = Some(e);
                return;
            }
        };
        let engine::State::EfiLoader { simulation } = &self.progress.attempts[&source.id].state
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
        self.metrics.efi_loader = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    record(&mut self.metrics.efi_loader, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        next.view = v;
                        view = v;
                        self.lesson_scroll_reset = true;
                        self.focus_input = true;
                        self.notice = None;
                    }
                }
            });
        }
        ui.push_id((&source.id, view as u8, challenge), |ui| {
            if view == teaching::View::Introduction {
                lesson_presentation::concepts(
                    ui,
                    &source,
                    "Review GPT partitions, mounted directories and GRUB menus before this recovery assignment.",
                    std::iter::empty(),
                );
                lesson_presentation::worked(ui, &definition.example.instructions);
                if let Some(content) = lesson_presentation::entry(&source) {
                    for row in &content.worked {
                        ui.strong(&row.label);
                        lesson_presentation::monospace(ui, &row.value, true);
                        lesson_presentation::prose(ui, &row.explanation);
                        ui.add_space(controls::GAP);
                    }
                }
                ui.horizontal_wrapped(|ui| {
                    for (view, key, label) in [
                        (teaching::View::Example, "try", "Try this example"),
                        (teaching::View::Assignment, "start", "Open full assignment"),
                    ] {
                        let response = ui.add_enabled(active, button(label));
                        record(&mut self.metrics.efi_loader, key, &response, ui);
                        if response.clicked() {
                            next.view = view;
                            self.lesson_scroll_reset = true;
                            self.focus_input = true;
                            self.notice = None;
                        }
                    }
                });
                return;
            }

            let practice = view == teaching::View::Example;
            let (task, draft) = if practice {
                (&definition.example, &mut next.practice)
            } else {
                (&definition.initial, &mut next.assignment)
            };
            if practice {
                lesson_presentation::lead(ui, "PRACTICE TASK", &task.instructions);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            ui.add_enabled_ui(active, |ui| {
                let split = ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 48.;
                self.metrics.efi_loader.side_by_side = split;
                let mut controls = ConsoleControls {
                    metrics: &mut self.metrics.efi_loader,
                    focus: &mut self.focus_input,
                    notice: &mut self.notice,
                };
                if split {
                    ui.columns(2, |columns| {
                        console(&mut columns[0], draft, task, (&source, practice), &mut controls);
                        facts(&mut columns[1], task, draft);
                    });
                } else {
                    facts(ui, task, draft);
                    ui.add_space(controls::GAP);
                    console(ui, draft, task, (&source, practice), &mut controls);
                }
                controls::collapsing_header(ui, "Supported commands and hints · optional").show(ui, |ui| {
                    lesson_presentation::prose(ui, core::HELP);
                    for hint in &source.hints {
                        lesson_presentation::prose(ui, hint);
                    }
                });
                if !practice {
                    return;
                }
                controls::collapsing_header(ui, "All practice actions · optional").show(ui, |ui| {
                    for step in 0..4 {
                        lesson_presentation::practice_step(ui, &source, step);
                    }
                });
                let worked = controls::collapsing_header(ui, "Worked practice · optional").show(ui, |ui| {
                    for (index, command) in task.reference.iter().filter(|command| command.as_str() != work::CHECK_ACTION).enumerate() {
                        lesson_presentation::monospace(ui, command, true);
                        let response = ui.add(button("Insert command"));
                        record(&mut self.metrics.efi_loader, &format!("insert-{index}"), &response, ui);
                        if response.clicked() {
                            draft.command = command.clone();
                            self.focus_input = true;
                        }
                    }
                    lesson_presentation::prose(
                        ui,
                        if task.media.is_some() {
                            "Insertion fills the practice input. Run command applies it. Test recovery paths checks both startup conditions."
                        } else {
                            "Insertion fills the practice input. Run command applies it. Test boot chain checks the resulting state."
                        },
                    );
                });
                record(&mut self.metrics.efi_loader, "worked", &worked.header_response, ui);
            });
        });
        if let Some((rect, id)) = self.metrics.efi_loader.input {
            self.metrics.focus = Some(id);
            self.metrics.editor_viewport = Some(rect);
        }
        self.metrics.check = self
            .metrics
            .efi_loader
            .controls
            .iter()
            .find(|r| r.0 == "test")
            .map(|r| r.1);
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::EfiLoader {
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
