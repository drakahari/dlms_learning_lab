//! Supplied-machine BIOS recovery console; every command is interpreted in process.
use super::*;
use dlms_learning_lab::{bios_loader as core, bios_loader_workspace as work};
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
fn facts(ui: &mut egui::Ui, t: &work::Task) {
    ui.strong("Supplied machine · BIOS / MBR");
    lesson_presentation::monospace(ui, &format!("BIOS starts from {}", t.facts.boot_disk), true);
    lesson_presentation::prose(
        ui,
        &format!(
            "The supplied i386-pc core needs {} sectors after the MBR. Mounted filesystems are already available; no setup on your computer is needed.",
            t.facts.core_sectors
        ),
    );
    for (name, disk) in &t.facts.geometry.disks {
        let space = t.facts.embedding_sectors(name).map_or_else(
            |e| e.to_string(),
            |n| format!("{n} embedding sectors available"),
        );
        lesson_presentation::monospace(ui, &format!("{name} · {space}"), true);
        for p in disk.partitions() {
            lesson_presentation::prose(
                ui,
                &format!(
                    "Partition {}: sectors {}–{}; existing contents stay intact.",
                    p.number, p.start, p.end
                ),
            );
        }
    }
    ui.add_space(controls::GAP);
    ui.strong("Mounted boot directories");
    for d in &t.facts.directories {
        lesson_presentation::monospace(
            ui,
            &format!("{} → {}{}", d.path, d.disk, d.partition),
            true,
        );
        lesson_presentation::prose(
            ui,
            if d.configuration.is_some() {
                "Contains supplied grub/grub.cfg and its image/root facts."
            } else {
                "No grub/grub.cfg supplied here. Installation alone does not create a menu."
            },
        );
    }
    if !t.preserve.is_empty() {
        ui.strong("Preserve other loaders");
        lesson_presentation::monospace(ui, &t.preserve.join(", "), true);
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
        lesson_presentation::practice_step(ui, s, 0);
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
        let test = ui.add(button("Test boot chain · simulation"));
        record(m, "test", &test, ui);
        if test.clicked() {
            **notice = draft.test(t).err();
        }
    });
    if let Some(error) = notice.as_deref() {
        ui.strong("Input needs attention");
        lesson_presentation::prose(ui, error);
    }
    if practice {
        lesson_presentation::practice_step(ui, s, 1);
    }
    if draft.current.current_test() {
        let rows = t.checks(&draft.current);
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
        // Failed requirements precede successful stages, including preservation:
        // a good handoff must not hide damage to another loader.
        for (_, message) in rows.iter().filter(|r| !r.0) {
            lesson_presentation::prose(ui, &format!("Needs attention · {message}"));
        }
        if passed {
            lesson_presentation::prose(
                ui,
                "The selected disk reaches the requested system, and the other loader remains intact.",
            );
        }
        controls::collapsing_header(ui, "Verified boot stages · optional").show(ui, |ui| {
            for (_, message) in rows.iter().filter(|r| r.0) {
                lesson_presentation::prose(ui, &format!("Verified · {message}"));
            }
        });
    } else {
        lesson_presentation::prose(
            ui,
            "Not tested for the current installation. Use Test boot chain after making a repair.",
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
    if practice {
        lesson_presentation::practice_step(ui, s, 2);
    }
}
impl LabApp {
    pub(super) fn bios_loader_page(&mut self, ui: &mut egui::Ui, i: usize) {
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
        let engine::State::BiosLoader { simulation } = &self.progress.attempts[&source.id].state
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
        self.metrics.bios_loader = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == v));
                    record(&mut self.metrics.bios_loader, key, &r, ui);
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
        ui.push_id((&source.id,view as u8,challenge),|ui|{
            if view==teaching::View::Introduction{
                lesson_presentation::concepts(ui,&source,"Review whole disks, partitions and a GRUB menu before this recovery assignment.",std::iter::empty());
                lesson_presentation::worked(ui,&definition.example.instructions);
                if let Some(content)=lesson_presentation::entry(&source){for(n,row)in content.worked.iter().enumerate(){ui.strong(&row.label);lesson_presentation::code_value(ui,("bios-worked",n),&row.value,ui.available_width());lesson_presentation::prose(ui,&row.explanation);ui.add_space(controls::GAP);}}
                ui.horizontal_wrapped(|ui|{for(v,key,label)in[(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{let r=ui.add_enabled(active,button(label));record(&mut self.metrics.bios_loader,key,&r,ui);if r.clicked(){next.view=v;self.lesson_scroll_reset=true;self.focus_input=true;self.notice=None;}}});
            }else{
                let practice=view==teaching::View::Example;let task=if practice{&definition.example}else{&definition.initial};let draft=if practice{&mut next.practice}else{&mut next.assignment};
                if practice{lesson_presentation::lead(ui,"PRACTICE TASK",&task.instructions);}else{lesson_presentation::assignment(ui,&source);}
                ui.add_enabled_ui(active,|ui|{
                    self.metrics.bios_loader.side_by_side=ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*48.;
                    if self.metrics.bios_loader.side_by_side{ui.columns(2,|cols|{console(&mut cols[0],draft,task,(&source,practice),&mut ConsoleControls{metrics:&mut self.metrics.bios_loader,focus:&mut self.focus_input,notice:&mut self.notice});facts(&mut cols[1],task);});}else{facts(ui,task);ui.add_space(controls::GAP);console(ui,draft,task,(&source,practice),&mut ConsoleControls{metrics:&mut self.metrics.bios_loader,focus:&mut self.focus_input,notice:&mut self.notice});}
                    controls::collapsing_header(ui,"Supported commands and hints · optional").show(ui,|ui|{lesson_presentation::prose(ui,core::HELP);for hint in &source.hints{lesson_presentation::prose(ui,hint);}});
                    if practice{let opened=controls::collapsing_header(ui,"Worked practice · optional").show(ui,|ui|{
                        for(n,command)in task.reference.iter().filter(|c|c.as_str()!=work::CHECK_ACTION).enumerate(){lesson_presentation::monospace(ui,command,true);let r=ui.add(button("Insert command"));record(&mut self.metrics.bios_loader,&format!("insert-{n}"),&r,ui);if r.clicked(){draft.command=command.clone();self.focus_input=true;}}
                        lesson_presentation::prose(ui,"Insertion only fills this practice input. Run command applies it; Test boot chain checks the resulting state.");
                    });record(&mut self.metrics.bios_loader,"worked",&opened.header_response,ui);}
                });
            }
        });
        if let Some((rect, id)) = self.metrics.bios_loader.input {
            self.metrics.focus = Some(id);
            self.metrics.editor_viewport = Some(rect);
        }
        self.metrics.check = self
            .metrics
            .bios_loader
            .controls
            .iter()
            .find(|r| r.0 == "test")
            .map(|r| r.1);
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::BiosLoader {
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
