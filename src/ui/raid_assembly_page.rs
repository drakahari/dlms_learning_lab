//! Native saved-policy editor and existing-mirror investigation terminal.
use super::*;
use dlms_learning_lab::{raid_assembly_source as binding, raid_assembly_workspace as work};
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id, egui::Rect)>,
    pub result: Option<(usize, usize, egui::Rect)>,
    pub goal: Option<egui::Rect>,
    pub action_notice: Option<egui::Rect>,
}
fn record(m: &mut Metrics, key: impl Into<String>, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
// Two ephemeral slots, one per role. No profile field or grading authority.
#[derive(Clone)]
struct ActionFeedback {
    source: String,
    version: u32,
    visit: u64,
    message: String,
}
fn feedback(
    ui: &mut egui::Ui,
    source: &Scenario,
    visit: u64,
    practice: bool,
    changed: bool,
    attempted: bool,
    error: Option<String>,
) -> Option<egui::Rect> {
    let key = egui::Id::new(("raid_assembly-action-feedback", practice));
    let prior = ui
        .ctx()
        .data(|d| d.get_temp::<ActionFeedback>(key))
        .filter(|n| n.source == source.id && n.version == source.version && n.visit == visit);
    let reveal = error.is_some();
    let next = if let Some(message) = error {
        let message = if message.starts_with("Unsupported command or path.") {
            "Unsupported command or path. Nothing ran; use the supplied mdadm inspect/assemble/stop forms or cat /proc/mdstat. Full supported forms are in Help and simulation scope.".into()
        } else {
            message
        };
        Some(ActionFeedback {
            source: source.id.clone(),
            version: source.version,
            visit,
            message,
        })
    } else if changed || attempted {
        None
    } else {
        prior
    };
    let response = next.as_ref().map(|n| {
        let r = lesson_presentation::prose(ui, &format!("Needs attention · {}", n.message));
        if reveal {
            r.scroll_to_me(Some(egui::Align::Center));
        }
        r.rect
    });
    ui.ctx().data_mut(|d| {
        if let Some(next) = next {
            d.insert_temp(key, next);
        } else {
            d.remove::<ActionFeedback>(key);
        }
    });
    response
}
fn worked(ui: &mut egui::Ui, step: &work::Step) {
    match &step.operation {
        work::Operation::Command { text } => {
            lesson_presentation::command(ui, text, &step.explanation)
        }
        work::Operation::Save { text } => {
            ui.strong("Example file · /etc/mdadm.conf");
            lesson_presentation::monospace(ui, &text.0, true);
            lesson_presentation::prose(ui, &step.explanation);
        }
    }
    if step.status != 0 {
        ui.label("Expected result · failure");
    }
}
impl LabApp {
    pub(super) fn raid_assembly_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let source = self.scenarios[i].clone();
        let packet = ui.input(|i| {
            i.events
                .iter()
                .any(|e| matches!(e, egui::Event::Text(_) | egui::Event::Paste(_)))
        });
        let held_key = egui::Id::new(("raid_assembly-held-input", &source.id));
        let old_held = ui
            .ctx()
            .data(|d| d.get_temp::<bool>(held_key))
            .unwrap_or(false);
        let down = ui.input(|i| i.pointer.primary_down());
        let new_held = down && (old_held || packet);
        ui.ctx().data_mut(|d| d.insert_temp(held_key, new_held));
        let action_enabled = !packet && !old_held;
        let phase = (self.selected, self.reset_pending);
        let challenge = ui
            .add_enabled_ui(action_enabled, |ui| self.workspace_header(ui, i))
            .inner;
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        if let Err(e) = binding::context(&self.progress.attempts[&source.id], &source) {
            ui.label(e);
            return;
        }
        let engine::Goal::RaidAssembly { definition: d } = &source.goal else {
            return;
        };
        let engine::State::RaidAssembly { simulation } = &self.progress.attempts[&source.id].state
        else {
            return;
        };
        let visited = self.progress.attempts[&source.id].visited;
        let mut s = (**simulation).clone();
        let before = s.clone();
        let active = ui.is_enabled()
            && !self.reset_pending
            && !self.settings_open
            && !self.theme_preview
            && !self.data_tools.modal_open();
        let enabled = active && action_enabled;
        let view = if challenge {
            teaching::View::Assignment
        } else {
            s.view
        };
        lesson_presentation::rich(ui, RichText::new(&source.title).strong().size(22.));
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(enabled, egui::Button::new(label).selected(view == v));
                    record(&mut self.metrics.raid_assembly, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        s.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        let practice = view == teaching::View::Example;
        let mut error = None;
        let mut attempted = false;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "After RAID1 member recovery. Existing mirror data and basic metadata are explained here before assembly practice.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "Follow this separate example before the assignment. Save file writes only the virtual configuration; mdadm commands act on supplied simulated members.",
            );
            if let Some(entry) = lesson_presentation::entry(&source) {
                for row in &entry.worked {
                    ui.strong(&row.label);
                    lesson_presentation::monospace(ui, &row.value, true);
                    lesson_presentation::prose(ui, &row.explanation);
                    ui.add_space(4.);
                }
            }

            for step in &d.example_steps {
                worked(ui, step);
                ui.add_space(4.);
            }
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(enabled, button(label));
                    record(&mut self.metrics.raid_assembly, key, &r, ui);
                    if r.clicked() {
                        s.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
                let picker = controls::dropdown(
                    ui,
                    ("raid_assembly-example-choice", &source.id, visited),
                    ui.available_width(),
                )
                .selected_text(format!(
                    "{}. {}",
                    s.example_step + 1,
                    d.example_steps[s.example_step].operation.label()
                ))
                .show_ui(ui, |ui| {
                    for (n, step) in d.example_steps.iter().enumerate() {
                        let r = ui.selectable_value(
                            &mut s.example_step,
                            n,
                            format!("{}. {}", n + 1, step.operation.label()),
                        );
                        record(
                            &mut self.metrics.raid_assembly,
                            format!("example-choice-{n}"),
                            &r,
                            ui,
                        );
                    }
                });
                record(
                    &mut self.metrics.raid_assembly,
                    "example-picker",
                    &picker.response,
                    ui,
                );
                let step = &d.example_steps[s.example_step];
                worked(ui, step);
                let label = match step.operation {
                    work::Operation::Save { .. } => "Fill selected example",
                    work::Operation::Command { .. } => "Insert command",
                };
                {
                    let r = ui.add_enabled(enabled, button(label));
                    record(&mut self.metrics.raid_assembly, "insert", &r, ui);
                    if r.clicked() {
                        match &step.operation {
                            work::Operation::Save { text } => {
                                if let Err(e) = s.edit(d, text.0.clone(), true) {
                                    error = Some(e);
                                }
                            }
                            work::Operation::Command { text } => {
                                s.practice.command = text.clone();
                                self.focus_input = true;
                            }
                        }
                    }
                }
                lesson_presentation::practice_step(ui, &source, 1);
            } else {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics.raid_assembly.goal = Some(r.rect);
            }
            let a = work::Session::activity(d, practice);
            ui.add_space(8.);
            ui.strong("Configuration · /etc/mdadm.conf (simulated)");
            let id = egui::Id::new((
                "raid_assembly-editor",
                &source.id,
                source.version,
                visited,
                practice,
            ));
            let previous = egui::text_edit::TextEditState::load(ui.ctx(), id).map(|state| {
                let undo = state.undoer();
                (state, undo)
            });
            let edit = shared_libraries_page::configuration_input(
                ui,
                &s.work(practice).draft.0,
                id,
                4096,
                active,
                "DEVICE literal-components and ARRAY /dev/mdN UUID=identity",
            );
            self.metrics.raid_assembly.fields.push((
                "editor".into(),
                edit.response.rect,
                id,
                edit.clip,
            ));
            let mut rejected = edit.rejected;
            if let Some(text) = edit.value
                && let Err(e) = s.edit(d, text, practice)
            {
                rejected = true;
                error = Some(e);
                if let Some((mut state, undo)) = previous {
                    state.set_undoer(undo);
                    state.store(ui.ctx(), id);
                }
            }
            if rejected && error.is_none() {
                error=Some("This edit exceeds the supported text format. The previous draft, saved file and active arrays are retained; nothing ran.".into());
            }
            let actions = enabled && !rejected;
            ui.horizontal_wrapped(|ui| {
                let r = ui.add_enabled(actions, button("Save file"));
                record(&mut self.metrics.raid_assembly, "save", &r, ui);
                attempted |= r.clicked();
                if r.clicked()
                    && let Err(e) = s.save(d, practice)
                {
                    error = Some(e);
                }
                if s.work(practice).draft != s.work(practice).runtime.saved {
                    ui.label("Unsaved edits · choose Save file before checking");
                }
            });
            ui.add_space(6.);
            let data_panel = controls::collapsing_header(ui, "Original sample data · simulator inspection").show(ui, |ui| {
                ui.label("This reads retained replica content through active identities; no real filesystem is mounted.");
                for requirement in &a.requirements {
                    lesson_presentation::monospace(ui, &format!("{} · {}", requirement.target.0, requirement.file.0), true);
                    match a.context.read(&s.work(practice).runtime.live, &requirement.target.0, &requirement.file.0) {
                        Ok(text) => { lesson_presentation::monospace(ui, &text, true); }
                        Err(e) => { lesson_presentation::prose(ui, &e); }
                    }
                }
            });
            record(
                &mut self.metrics.raid_assembly,
                "data",
                &data_panel.header_response,
                ui,
            );
            let empty = s.work(practice).transcript.is_empty();
            let height = (ui.ctx().content_rect().height() * 0.23).clamp(120., 250.);
            let output=egui::Frame::new().fill(palette(ui).input).inner_margin(10.).show(ui,|ui|{
                ui.strong(if practice{"Actual example terminal"}else{"Simulated assignment terminal"});
                if empty {
                    lesson_presentation::prose(ui,"Inspect original members with mdadm --examine, inspect the configuration with cat /etc/mdadm.conf, and inspect active arrays with cat /proc/mdstat.");
                } else {
                    controls::nested_scroll(ui,egui::ScrollArea::both().id_salt(("raid_assembly-output",&source.id,visited,practice)).auto_shrink([false,false]).min_scrolled_height(height).max_height(height).stick_to_bottom(true),|ui|{
                        for row in &s.work(practice).transcript{lesson_presentation::monospace(ui,&format!("{}{}\n{}",if row.action.starts_with("Save "){""}else{"$ "},row.action,row.result.display()),false);}
                    });
                }
            });
            self.metrics.output = Some(output.response.rect);
            let id = egui::Id::new((
                "raid_assembly-command",
                &source.id,
                source.version,
                visited,
                practice,
            ));
            let mut run = false;
            ui.horizontal(|ui|{ui.monospace("$");let width=(ui.available_width()-controls::button_width(ui,"Run")-ui.spacing().item_spacing.x).max(40.);
                let edit=ui.allocate_ui_with_layout(egui::vec2(width,controls::height(ui)),egui::Layout::top_down(egui::Align::Min),|ui|shared_libraries_page::text_input(ui,&s.work(practice).command,id,2048,false,active,"e.g. cat /proc/mdstat")).inner;
                self.metrics.inputs.push(edit.response.rect);self.metrics.raid_assembly.fields.push(("command".into(),edit.response.rect,id,edit.clip));
                if let Some(text)=edit.value{s.work_mut(practice).command=text;}
                if edit.rejected{rejected=true;error=Some("Use one command per run. No pasted script or newline was executed; previous input retained.".into());}
                if self.focus_input&&active&&!packet{edit.response.request_focus();self.focus_input=false;}
                run=actions&&!rejected&&(edit.response.has_focus()||edit.response.lost_focus())&&ui.input(|i|i.key_pressed(egui::Key::Enter));
                let r=ui.add_enabled(actions&&!rejected,button("Run"));record(&mut self.metrics.raid_assembly,"run",&r,ui);run|=r.clicked();
            });
            if run {
                attempted = true;
                let text = s.work(practice).command.clone();
                if let Err(e) = s.apply(d, &work::Operation::Command { text }, practice) {
                    error = Some(e);
                }
                self.focus_input = true;
            }
            let r = ui.add_enabled(
                actions && !rejected,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check work"
                    },
                ),
            );
            record(&mut self.metrics.raid_assembly, "check", &r, ui);
            attempted |= r.clicked();
            self.metrics.check = Some(r.rect);
            if r.clicked()
                && let Err(e) = s.run(d, work::CHECK, practice)
            {
                error = Some(e);
            }
            if s.work(practice).checked {
                let rows = s.work(practice).checks(a);
                let failed = rows.iter().filter(|r| !r.0).count();
                let r = ui.strong(if failed == 0 {
                    format!("Passed · all {} objective checks", rows.len())
                } else {
                    format!("Needs attention · {failed} remaining checks")
                });
                self.metrics.raid_assembly.result = Some((rows.len(), failed, r.rect));
                let mut causes = std::collections::BTreeSet::new();
                let unique = rows
                    .iter()
                    .filter(|r| !r.0)
                    .filter(|r| {
                        let cause = r.1.rsplit_once(": ").map_or(r.1.as_str(), |(_, tail)| tail);
                        causes.insert(cause)
                    })
                    .take(3)
                    .collect::<Vec<_>>();
                for (_, text) in unique {
                    lesson_presentation::prose(ui, &format!("Needs attention · {text}"));
                }
                controls::collapsing_header(ui, "All objective results").show(ui, |ui| {
                    for (ok, text) in &rows {
                        lesson_presentation::prose(
                            ui,
                            &format!(
                                "{} · {text}",
                                if *ok { "Passed" } else { "Needs attention" }
                            ),
                        );
                    }
                });
                if failed == 0 && !practice {
                    lesson_presentation::prose(ui, &source.explanation);
                }
            } else {
                ui.label("Not checked · saved assembly policy, active identity and original data are evaluated separately");
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                worked(ui, &d.experiment);
                let r = ui.add_enabled(actions && !rejected, button("Reset example only"));
                record(&mut self.metrics.raid_assembly, "reset-example", &r, ui);
                if r.clicked() {
                    s.practice = work::Work::new(&d.example).unwrap();
                    s.example_step = 0;
                }
            }
            controls::collapsing_header(ui, "Help and simulation scope").show(ui, |ui| {
                lesson_presentation::prose(ui, work::HELP);
                if !practice {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    let r = ui.add_enabled(
                        actions && !rejected && a.hints < source.hints.len(),
                        button("Show a hint"),
                    );
                    record(&mut self.metrics.raid_assembly, "hint", &r, ui);
                    if r.clicked() {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for h in source.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, h);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals a solution").show(
                        ui,
                        |ui| {
                            for step in &d.reference {
                                worked(ui, step);
                            }
                        },
                    );
                }
                lesson_presentation::prose(ui, &source.simplifications);
                for link in &source.sources {
                    ui.hyperlink_to(link, link.split_whitespace().next().unwrap_or(link));
                }
            });
        }
        self.metrics.raid_assembly.action_notice = feedback(
            ui,
            &source,
            visited,
            practice,
            s != before,
            attempted,
            error,
        );
        if s != before && active {
            match d.bound(&s) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::RaidAssembly {
                            simulation: Box::new(s),
                        };
                    self.completion_index.forget(&source.id);
                    self.dirty = true;
                    ui.ctx().request_repaint();
                }
                Err(e) => self.notice = Some(e),
            }
        }
        if view == teaching::View::Assignment
            && progress::current_solved(&self.progress.attempts[&source.id], &source)
        {
            self.next_unit_lab(ui, i);
        }
    }
}
