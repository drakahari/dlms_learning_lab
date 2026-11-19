//! Native mirrored-storage investigation. Every operation is in-process and bounded.
use super::*;
use dlms_learning_lab::{engine::State, raid1 as core, raid1_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub command: Option<(egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, name: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((name.into(), r.rect, ui.clip_rect()));
}
fn capacity(sectors: u64) -> String {
    format!(
        "{}{} KiB",
        sectors / 2,
        if sectors.is_multiple_of(2) { "" } else { ".5" }
    )
}
fn member_role(r: core::Role) -> &'static str {
    match r {
        core::Role::Synced => "Synchronized",
        core::Role::Faulty => "Failed · still attached",
        core::Role::Spare => "Idle spare",
        core::Role::Recovering { .. } => "Copying · not synchronized",
    }
}
fn facts(ui: &mut egui::Ui, world: &core::State, target: &str) {
    let a = &world.arrays[target];
    let health = world.health(target).unwrap();
    lesson_presentation::rich(
        ui,
        RichText::new(format!("{target} · {}", health.label())).strong(),
    );
    lesson_presentation::prose(
        ui,
        &format!(
            "Two-member mirror · required usable capacity per member: {}",
            capacity(a.component_sectors)
        ),
    );
    if let core::Health::Rebuilding { copied, total } = health {
        ui.add(
            egui::ProgressBar::new(f32::from(copied) / f32::from(total))
                .text(format!("Actual sample segments copied: {copied}/{total}")),
        );
    }
    let rows = (0..2)
        .map(|slot| {
            let member = a
                .members
                .iter()
                .find(|m| m.slot == Some(slot) && m.role != core::Role::Faulty)
                .or_else(|| a.members.iter().find(|m| m.slot == Some(slot)));
            (
                format!("Member slot {}", slot),
                member
                    .map(|m| format!("{} · {}", m.device, member_role(m.role)))
                    .unwrap_or_else(|| "Missing · no attached member".into()),
            )
        })
        .collect::<Vec<_>>();
    for (label, value) in rows {
        controls::form_row(ui, &label, |ui, _| {
            lesson_presentation::monospace(ui, &value, true);
        });
    }
    for m in a.members.iter().filter(|m| {
        m.role == core::Role::Faulty
            && a.members
                .iter()
                .any(|other| other.role != core::Role::Faulty && other.slot == m.slot)
    }) {
        lesson_presentation::monospace(
            ui,
            &format!(
                "{} · Failed member still attached; remove it after confirming its identity",
                m.device
            ),
            true,
        );
    }
    for m in a.members.iter().filter(|m| m.role == core::Role::Spare) {
        lesson_presentation::monospace(ui, &format!("{} · Idle spare", m.device), true);
    }
    ui.add_space(6.);
    lesson_presentation::rich(ui, RichText::new("Replacement inventory").strong());
    let candidates = world
        .devices
        .iter()
        .filter(|(path, _)| {
            !world
                .arrays
                .values()
                .any(|a| a.members.iter().any(|m| m.device == **path))
        })
        .collect::<Vec<_>>();
    let describe = |d: &core::Device| {
        let mut flags = vec![];
        if d.protected {
            flags.push("Protected");
        }
        if !d.available {
            flags.push("Unavailable");
        }
        if d.read_only {
            flags.push("Read-only");
        }
        match &d.media {
            core::Media::Blank => flags.push("Blank"),
            core::Media::Foreign => flags.push("Contains other data"),
            core::Media::Md { .. } => flags.push("Retained MD metadata"),
        }
        if d.usable_sectors < a.component_sectors {
            flags.push("Too small");
        }
        flags.join(" · ")
    };
    if ui.available_width() >= ui.text_style_height(&egui::TextStyle::Body) * 38. {
        egui::Grid::new(("raid1-inventory", target))
            .num_columns(3)
            .spacing(egui::vec2(16., 6.))
            .show(ui, |ui| {
                for h in ["Supplied device", "Usable MD capacity", "Condition"] {
                    ui.strong(h);
                }
                ui.end_row();
                for (path, d) in &candidates {
                    ui.monospace(*path);
                    ui.label(capacity(d.usable_sectors));
                    ui.add(egui::Label::new(describe(d)).wrap());
                    ui.end_row();
                }
            });
    } else {
        for (path, d) in &candidates {
            lesson_presentation::monospace(ui, path, true);
            lesson_presentation::prose(
                ui,
                &format!("{} · {}", capacity(d.usable_sectors), describe(d)),
            );
            ui.add_space(4.);
        }
    }
    controls::collapsing_header(ui, "Other supplied arrays and retained data · read-only").show(
        ui,
        |ui| {
            for (name, array) in &world.arrays {
                if name != target {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!("{name} · {}", world.health(name).unwrap().label()))
                            .strong(),
                    );
                    for member in &array.members {
                        lesson_presentation::monospace(
                            ui,
                            &format!("{} · {}", member.device, member_role(member.role)),
                            true,
                        );
                    }
                }
                let content = world.replica_content(name).unwrap();
                let files = content
                    .iter()
                    .map(|s| s.file.as_str())
                    .collect::<std::collections::BTreeSet<_>>();
                for file in files {
                    ui.strong(format!("{name}:{file}"));
                    lesson_presentation::monospace(ui, &world.read_file(name, file).unwrap(), true);
                }
            }
        },
    );
}
struct CommandBuffer<'a> {
    value: &'a mut String,
    rejected: bool,
    received: bool,
}
impl egui::TextBuffer for CommandBuffer<'_> {
    fn type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<CommandBuffer<'static>>()
    }
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_str(&self) -> &str {
        self.value
    }
    fn insert_text(&mut self, text: &str, at: usize) -> usize {
        if text.is_empty() || self.rejected {
            return 0;
        }
        self.received = true;
        if text.contains(['\0', '\n', '\r'])
            || self
                .value
                .len()
                .checked_add(text.len())
                .is_none_or(|n| n > 512)
        {
            self.rejected = true;
            return 0;
        }
        egui::TextBuffer::insert_text(self.value, text, at)
    }
    fn delete_char_range(&mut self, r: std::ops::Range<usize>) {
        if !self.rejected {
            egui::TextBuffer::delete_char_range(self.value, r)
        }
    }
}
impl LabApp {
    pub(super) fn raid1_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let before = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if before != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(attempt, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let (State::Raid1 { simulation }, engine::Goal::Raid1 { definition }) =
            (&attempt.state, &source.goal)
        else {
            return;
        };
        let original = (**simulation).clone();
        let mut next = original.clone();
        let visited = attempt.visited;
        let allowed = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let mut view = if challenge {
            teaching::View::Assignment
        } else {
            next.view
        };
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Introduction, "Introduction"),
                    (teaching::View::Example, "Try example · ungraded"),
                    (teaching::View::Assignment, "Assignment"),
                ] {
                    let r = ui.selectable_label(view == v, label);
                    self.metrics.lesson_tabs.push(r.rect);
                    record(&mut self.metrics.raid1, &format!("view:{v:?}"), &r, ui);
                    if r.clicked() {
                        view = v;
                    }
                }
            });
            if view != next.view
                && let Err(e) = next.set_view(definition, view)
            {
                self.notice = Some(e);
                return;
            }
        }
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                &definition.lesson.prerequisite_note,
                definition
                    .lesson
                    .concepts
                    .iter()
                    .map(|c| (c.title.clone(), c.text.clone())),
            );
            lesson_presentation::worked(ui, &definition.lesson.description);
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, label) in [
                    (teaching::View::Example, "Try this example"),
                    (teaching::View::Assignment, "Go straight to assignment"),
                ] {
                    let r = ui.add(button(label));
                    record(
                        &mut self.metrics.raid1,
                        if v == teaching::View::Example {
                            "start-example"
                        } else {
                            "start-assignment"
                        },
                        &r,
                        ui,
                    );
                    if r.clicked() {
                        let _ = next.set_view(definition, v);
                    }
                }
            });
        } else {
            let practice = view == teaching::View::Example;
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let target = if practice {
                &definition.lesson.plan.array
            } else {
                &definition.plan.array
            };
            let world = if practice {
                &next.example
            } else {
                &next.current
            };
            facts(ui, world, target);
            if practice {
                lesson_presentation::practice_step(ui, &source, 1);
                let mut selected = next.selected_example;
                let r = controls::dropdown(
                    ui,
                    ("raid1-guided", &source.id, visited),
                    ui.available_width(),
                )
                .selected_text(&definition.lesson.worked[selected].title)
                .show_ui(ui, |ui| {
                    controls::fit_popup_contents(ui);
                    let start = self.metrics.raid1.controls.len();
                    for (n, s) in definition.lesson.worked.iter().enumerate() {
                        let r = ui.selectable_value(&mut selected, n, &s.title);
                        record(&mut self.metrics.raid1, &format!("example:{n}"), &r, ui);
                    }
                    let clip = controls::popup_content_clip(ui);
                    for row in &mut self.metrics.raid1.controls[start..] {
                        row.2 = clip;
                    }
                });
                record(&mut self.metrics.raid1, "example-picker", &r.response, ui);
                next.selected_example = selected;
                let step = &definition.lesson.worked[selected];
                lesson_presentation::prose(ui, &step.explanation);
                if let Some(command) = &step.command {
                    lesson_presentation::monospace(ui, command, true);
                    let r = ui.add_enabled(allowed, button("Insert command"));
                    record(&mut self.metrics.raid1, "insert", &r, ui);
                    if r.clicked()
                        && let Err(e) = next.set_command(definition, command.clone(), true)
                    {
                        self.notice = Some(e);
                    }
                }
            }
            ui.add_space(6.);
            lesson_presentation::rich(ui, RichText::new("Simulated RAID terminal").strong());
            let transcript = if practice {
                &next.example_transcript
            } else {
                &next.transcript
            };
            let height = ui.text_style_height(&egui::TextStyle::Monospace) * 6.;
            self.metrics.output = Some(if transcript.is_empty() {
                lesson_presentation::prose(
                    ui,
                    "No commands run yet. Inspect the array with mdadm --detail.",
                )
                .rect
            } else {
                let scroll = controls::nested_scroll(
                    ui,
                    egui::ScrollArea::vertical()
                        .id_salt(("raid1-output", &source.id, practice))
                        .max_height(height)
                        .min_scrolled_height(height)
                        .auto_shrink([false, false])
                        .stick_to_bottom(true),
                    |ui| {
                        if transcript.is_empty() {
                            lesson_presentation::prose(
                                ui,
                                "No commands run yet. Inspect the array with mdadm --detail.",
                            );
                        }
                        for row in transcript {
                            lesson_presentation::monospace(
                                ui,
                                &format!(
                                    "virtual $ {}\n{}\nExit status {}",
                                    row.input, row.output, row.status
                                ),
                                true,
                            );
                        }
                    },
                );
                scroll.inner_rect
            });
            let id = egui::Id::new((
                "raid1-command",
                &source.id,
                source.version,
                visited,
                practice,
            ));
            let packet = ui.input(|i| {
                i.events
                    .iter()
                    .any(|e| matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty()))
            });
            let invalid=ui.input(|i|i.events.iter().any(|e|matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if t.len()>512||t.contains(['\0','\n','\r']))));
            let after_enter = ui.input(|i| {
                let mut entered = false;
                i.events.iter().any(|e| {
                    if matches!(
                        e,
                        egui::Event::Key {
                            key: egui::Key::Enter,
                            pressed: true,
                            ..
                        }
                    ) {
                        entered = true
                    }
                    entered
                        && matches!(e,egui::Event::Text(t)|egui::Event::Paste(t)if !t.is_empty())
                })
            });
            let mut draft = if practice {
                next.example_command.clone()
            } else {
                next.command.clone()
            };
            let old = draft.clone();
            let previous = egui::text_edit::TextEditState::load(ui.ctx(), id).map(|s| {
                let undo = s.undoer();
                (s, undo)
            });
            let mut buf = CommandBuffer {
                value: &mut draft,
                rejected: invalid,
                received: false,
            };
            let r = ui.add_enabled(
                allowed,
                controls::singleline(ui, &mut buf)
                    .id(id)
                    .hint_text("mdadm --detail ARRAY")
                    .desired_width(ui.available_width()),
            );
            let rejected = allowed && (buf.rejected || (packet && !buf.received) || after_enter);
            self.metrics.raid1.command = Some((r.rect, id));
            record(&mut self.metrics.raid1, "command", &r, ui);
            if rejected {
                draft = old;
                if let Some((mut state, undo)) = previous {
                    state.set_undoer(undo);
                    state.store(ui.ctx(), id)
                }
                self.notice=Some("Nothing ran. Enter one supported command of up to512bytes without line breaks; the previous draft is retained.".into());
            } else if r.changed()
                && let Err(e) = next.set_command(definition, draft.clone(), practice)
            {
                self.notice = Some(e);
            }
            let enter = allowed
                && !rejected
                && r.lost_focus()
                && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let rebuilding = matches!(
                (if practice {
                    &next.example
                } else {
                    &next.current
                })
                .health(target),
                Ok(core::Health::Rebuilding { .. })
            );
            ui.horizontal_wrapped(|ui| {
                let run = ui.add_enabled(allowed && !rejected, button("Run command"));
                record(&mut self.metrics.raid1, "run", &run, ui);
                self.metrics.terminal_run = Some(run.rect);
                if run.clicked() || enter {
                    match next.execute(definition, &draft, practice) {
                        Ok(_) => self.notice = None,
                        Err(e) => self.notice = Some(e),
                    }
                }
                let advance = ui.add_enabled(
                    allowed && !rejected && rebuilding,
                    button("Advance simulated recovery"),
                );
                record(&mut self.metrics.raid1, "advance", &advance, ui);
                if advance.clicked() {
                    match next.action(
                        definition,
                        &core::Action::Advance {
                            array: target.clone(),
                            units: 4,
                        },
                        practice,
                    ) {
                        Ok(_) => self.notice = None,
                        Err(e) => self.notice = Some(e),
                    }
                }
                let check = ui.add_enabled(
                    allowed && !rejected,
                    button(if practice {
                        "Check example · ungraded"
                    } else {
                        "Check work"
                    }),
                );
                record(&mut self.metrics.raid1, "check", &check, ui);
                self.metrics.check = Some(check.rect);
                if check.clicked() {
                    if practice {
                        next.example_checked = true
                    } else {
                        next.checked = true
                    }
                    self.notice = None;
                }
            });
            let checked = if practice {
                next.example_checked
            } else {
                next.checked
            };
            if checked {
                let rows = if practice {
                    next.example_checks(definition)
                } else {
                    definition.checks(&next)
                };
                let passed = rows.iter().filter(|r| r.0).count();
                let success = passed == rows.len();
                let r = lesson_presentation::rich(
                    ui,
                    RichText::new(format!(
                        "{} · {passed}/{} requirements",
                        if success { "Passed" } else { "Needs work" },
                        rows.len()
                    ))
                    .strong()
                    .color(if success {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }),
                );
                self.metrics.raid1.result = Some((passed, rows.len(), r.rect));
                for (_, text) in rows.iter().filter(|r| !r.0) {
                    lesson_presentation::prose(ui, text);
                }
                controls::collapsing_header(ui, "Objective results").show(ui, |ui| {
                    for (pass, text) in &rows {
                        lesson_presentation::prose(
                            ui,
                            &format!("{}: {text}", if *pass { "Passed" } else { "Needs work" }),
                        );
                    }
                });
                if !practice && success {
                    self.metrics.completion = true;
                    lesson_presentation::prose(ui, &source.explanation);
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(button("Back to library"));
                        record(&mut self.metrics.raid1, "library", &r, ui);
                        if r.clicked() {
                            self.selected = None;
                        }
                    });
                    self.next_unit_lab(ui, i);
                }
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(allowed, button("Reset example only"));
                    record(&mut self.metrics.raid1, "reset-example", &r, ui);
                    if r.clicked()
                        && let Err(e) = next.reset_example(definition)
                    {
                        self.notice = Some(e)
                    }
                    let r = ui.add(button("Open full assignment"));
                    record(&mut self.metrics.raid1, "assignment", &r, ui);
                    if r.clicked() {
                        let _ = next.set_view(definition, teaching::View::Assignment);
                    }
                });
            }
            ui.push_id(("raid1-help",&source.id,source.version,visited,practice,challenge),|ui| {
                let help=controls::collapsing_header(ui,"Optional help and simulation limits").show(ui,|ui|{
                    if !practice {
                        let a=self.progress.attempts.get_mut(&source.id).unwrap();
                        let r=ui.add_enabled(allowed&&a.hints<source.hints.len(),button("Show a hint"));
                        record(&mut self.metrics.raid1,"hint",&r,ui);
                        if r.clicked(){a.hints+=1;self.dirty=true;}
                        for hint in source.hints.iter().take(a.hints){lesson_presentation::prose(ui,hint);}
                    }
                    let worked=controls::collapsing_header(ui,"Worked approach · reveals a solution").show(ui,|ui|{
                        lesson_presentation::prose(ui,&source.explanation);
                        for c in &source.solution {
                            if c.starts_with(work::PREFIX){lesson_presentation::prose(ui,"Use Advance simulated recovery until copying has finished; inspect actual health and retained files.");}
                            else{lesson_presentation::monospace(ui,c,true);}
                        }
                    });
                    record(&mut self.metrics.raid1,"worked-help",&worked.header_response,ui);
                    lesson_presentation::prose(ui,core::HELP);
                    controls::collapsing_header(ui,"References and boundaries").show(ui,|ui|{
                        for url in &source.sources{ui.hyperlink_to(url,url);}
                        lesson_presentation::prose(ui,&source.simplifications);
                    });
                });
                record(&mut self.metrics.raid1,"help",&help.header_response,ui);
            });
        }
        if next != original {
            if let Err(e) = definition.bound(&next) {
                self.notice = Some(e);
                return;
            }
            if let State::Raid1 { simulation } =
                &mut self.progress.attempts.get_mut(&source.id).unwrap().state
            {
                **simulation = next;
                self.dirty = true;
                self.completion_index.forget(&source.id);
            }
        }
    }
}
