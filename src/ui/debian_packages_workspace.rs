//! Real closed command spellings over the virtual script-free package model.
use super::*;
use dlms_learning_lab::{
    debian_packages::State as Simulation,
    debian_packages_teaching::Learning,
    debian_packages_terminal::{self as terminal, Session, session},
};
use std::borrow::Cow;
#[derive(Default)]
struct InputEffect {
    dirty: bool,
    run: bool,
    error: Option<String>,
}
impl LabApp {
    pub(super) fn debian_packages_view(&self, i: usize) -> teaching::View {
        let a = &self.progress.attempts[&self.scenarios[i].id];
        if a.challenge {
            teaching::View::Assignment
        } else {
            a.workspace
                .as_ref()
                .and_then(|w| w.debian_packages_learning.as_ref())
                .map_or(teaching::View::Assignment, |l| l.view)
        }
    }
    pub(super) fn set_debian_packages_view(&mut self, i: usize, view: teaching::View) {
        let w = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap()
            .workspace
            .as_mut()
            .unwrap();
        if let Some(l) = &mut w.debian_packages_learning {
            l.view = view;
        }
        self.checked = view == teaching::View::Assignment && w.checked;
        self.focus_input = view != teaching::View::Introduction;
        self.scroll_command = false;
        self.help_open = false;
        self.notice = None;
        self.lesson_scroll_reset = true;
        self.dirty = true;
    }
    pub(super) fn debian_packages_workspace(&mut self, ui: &mut egui::Ui, i: usize) {
        let view = self.debian_packages_view(i);
        let blocked = self.reset_pending
            || self.settings_open
            || self.theme_preview
            || self.data_tools.modal_open();
        let s = &self.scenarios[i];
        ui.heading(&s.title);
        ui.label(
            RichText::new(
                "Virtual Debian packages · Script-free simulation; no host packages or services.",
            )
            .small()
            .color(palette(ui).muted),
        );
        let challenge = self.progress.attempts[&s.id].challenge;
        let mut switch = None;
        if s.debian_packages_teaching.is_some() && !challenge {
            ui.add_enabled_ui(!blocked, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for (v, label) in [
                        (teaching::View::Introduction, "Introduction"),
                        (teaching::View::Example, "Try example · ungraded"),
                        (teaching::View::Assignment, "Assignment"),
                    ] {
                        let r = ui.selectable_label(v == view, label);
                        self.metrics.lesson_tabs.push(r.rect);
                        if r.clicked() {
                            switch = Some(v);
                        }
                    }
                })
            });
        }
        if let Some(v) = switch {
            self.set_debian_packages_view(i, v);
            return;
        }
        match view {
            teaching::View::Introduction => {
                let Some(l) = s.debian_packages_teaching.as_ref() else {
                    return;
                };
                ui.label(RichText::new(&l.prerequisite_note).color(palette(ui).muted));
                for c in &l.concepts {
                    ui.label(RichText::new(&c.title).strong());
                    ui.label(&c.text);
                }
                ui.label(RichText::new("A separate example to investigate").strong());
                ui.label(&l.example_description);
                ui.label("The terminal accepts the disclosed Debian command subset. Inspect records before changing them. Insert fills a command; only Run submits it.");
                ui.label(format!("Try a change: {}", l.try_change.description));
                let mut next = None;
                ui.add_enabled_ui(!blocked, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let r = ui.add(primary(ui, "Try this example"));
                        self.metrics.example_action = Some(r.rect);
                        if r.clicked() {
                            next = Some(teaching::View::Example);
                        }
                        let r = ui.add(button("Go straight to assignment"));
                        self.metrics.assignment_action = Some(r.rect);
                        if r.clicked() {
                            next = Some(teaching::View::Assignment);
                        }
                    })
                });
                if let Some(v) = next {
                    self.set_debian_packages_view(i, v);
                }
            }
            teaching::View::Example => {
                let Some(lesson) = s.debian_packages_teaching.as_ref() else {
                    return;
                };
                ui.label("Practice example · Ungraded; assignment packages, command drafts and transcripts stay separate.");
                ui.label(&lesson.example_description);
                let l = self
                    .progress
                    .attempts
                    .get_mut(&s.id)
                    .unwrap()
                    .workspace
                    .as_mut()
                    .unwrap()
                    .debian_packages_learning
                    .as_mut()
                    .unwrap();
                let absent = l.terminal.is_none();
                if let Err(e) = l.initialize_terminal(lesson) {
                    ui.colored_label(palette(ui).error, e);
                    return;
                }
                self.dirty |= absent;
                let mut inserted = false;
                let mut reset = false;
                let mut assignment = false;
                ui.add_enabled_ui(!blocked,|ui|{
                    let before=l.selected_action;
                    let mut chosen=false;
                    ui.horizontal_wrapped(|ui|{
                        let button_width=controls::button_width(ui,"Insert command");
                        let available=ui.available_size_before_wrap().x;
                        let width=(available-button_width-controls::GAP).max(0.).min(available);
                        controls::cell(ui,width,|ui|{
                            let r=egui::ComboBox::from_id_salt(("package-example-action",&s.id)).width(width).truncate().selected_text(format!("Example action {} of {}",l.selected_action+1,lesson.example_actions.len())).show_ui(ui,|ui|{
                                for(n,a)in lesson.example_actions.iter().enumerate(){let option=ui.selectable_value(&mut l.selected_action,n,a.command.as_deref().unwrap_or("Read this example explanation"));self.metrics.package_example_items.push((n,option.rect));} self.metrics.package_example_popup=Some(controls::popup_content_clip(ui));
                            });
                            self.metrics.package_example=Some(r.response.rect);name_control(&r.response,"Ungraded package example action");
                        });
                        let r=ui.add_enabled(lesson.example_actions[l.selected_action].command.is_some(),button("Insert command"));self.metrics.command_insert=Some(r.rect);chosen=r.clicked();
                    });
                    self.dirty|=before!=l.selected_action;
                    let selected=&lesson.example_actions[l.selected_action];
                    ui.label(&selected.explanation);
                    if let Some(c)=&selected.command{ui.add(egui::Label::new(RichText::new(c).monospace()).wrap());}
                    ui.label(RichText::new("Insert changes the example command field only. Read it, then use Run command.").small().color(palette(ui).muted));
                    if chosen && let Some(command)=&selected.command{match l.insert_command(lesson,command){Ok(())=>{inserted=true;self.dirty=true;self.notice=None;},Err(e)=>self.notice=Some(e)}}
                    let old_command=l.terminal.as_ref().unwrap().command.clone();
                    let old_path=l.terminal.as_ref().unwrap().selected_path.clone();
                    let mut focus=self.focus_input&&!inserted;
                    let effects=terminal_panel(ui,(&s.id,true),&l.simulation,l.terminal.as_mut().unwrap(),&mut self.metrics,&mut focus);
                    if !inserted{self.focus_input=focus;}
                    let mut rejected_edit=false;
                    if effects.dirty{
                        if l.terminal_edits_valid(lesson){self.dirty=true;}else{rejected_edit=true;
                            let t=l.terminal.as_mut().unwrap();t.command=old_command;t.selected_path=old_path;
                            self.notice=Some("Practice input exceeds its bounded saved capacity; previous draft retained".into());
                        }
                    }
                    if let Some(e)=effects.error {self.notice=Some(e);} else if effects.dirty&&!rejected_edit {self.notice=None;}
                    if effects.run&&!rejected_edit{match l.submit_command(lesson){Ok(_)=>{self.dirty=true;self.notice=None;},Err(e)=>self.notice=Some(e)}}
                    let r=ui.add(primary(ui,"Check example · ungraded"));self.metrics.check=Some(r.rect);
                    if r.clicked(){l.checked=true;self.dirty=true;}
                    let feedback=controls::feedback(ui,r.clicked(),|ui|{
                        if l.checked{
                            let rows=l.checks(lesson);let solved=rows.iter().all(|r|r.0);
                            if solved{ui.colored_label(palette(ui).success,"Example checks pass · No assignment credit");}
                            current_rows(ui,&rows,(&s.id,true));
                        }
                    });self.metrics.feedback=Some(feedback.response.rect);
                    ui.label(format!("Try a change: {}",lesson.try_change.description));
                    ui.horizontal_wrapped(|ui|{
                        let r=ui.add(button("Reset example only"));self.metrics.example_reset=Some(r.rect);reset=r.clicked();
                        let r=ui.add(button("Open full assignment"));self.metrics.assignment_action=Some(r.rect);assignment=r.clicked();
                    });
                });
                if reset {
                    match Learning::new(lesson) {
                        Ok(mut new) => {
                            new.view = teaching::View::Example;
                            *l = new;
                            self.dirty = true;
                            self.notice = None;
                            self.focus_input = true;
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
                if inserted {
                    self.focus_input = true;
                }
                if assignment {
                    self.set_debian_packages_view(i, teaching::View::Assignment);
                }
            }
            teaching::View::Assignment => {
                ui.label(&s.objective);
                let engine::Goal::DebianPackages { definition } = &s.goal else {
                    return;
                };
                let a = self.progress.attempts.get_mut(&s.id).unwrap();
                let engine::State::DebianPackages { simulation } = &mut a.state else {
                    return;
                };
                let w = a.workspace.as_mut().unwrap();
                if w.debian_packages_terminal.is_none() {
                    w.debian_packages_terminal = Some(Session::default());
                    self.dirty = true;
                }
                let t = w.debian_packages_terminal.as_mut().unwrap();
                let effects = ui
                    .add_enabled_ui(!blocked, |ui| {
                        terminal_panel(
                            ui,
                            (&s.id, false),
                            simulation,
                            t,
                            &mut self.metrics,
                            &mut self.focus_input,
                        )
                    })
                    .inner;
                self.dirty |= effects.dirty;
                if let Some(e) = effects.error {
                    self.notice = Some(e);
                } else if effects.dirty {
                    self.notice = None;
                }
                if effects.run {
                    match t.submit(definition, simulation) {
                        Ok(changed) => {
                            if changed {
                                w.checked = false;
                                self.checked = false;
                            }
                            self.dirty = true;
                            self.notice = None;
                        }
                        Err(e) => self.notice = Some(e),
                    }
                }
                let r = ui.add_enabled(!blocked, primary(ui, "Check work"));
                self.metrics.check = Some(r.rect);
                if r.clicked() {
                    w.checked = true;
                    self.checked = true;
                    self.dirty = true;
                }
                let checked = w.checked;
                let mut complete = false;
                let result = controls::feedback(ui, r.clicked(), |ui| {
                    if checked {
                        let rows = progress::current_debian_packages_checks(
                            &self.progress.attempts[&s.id],
                            s,
                        );
                        complete = rows.iter().all(|r| r.0);
                        if complete {
                            ui.colored_label(
                                palette(ui).success,
                                "Objective complete · Current package and file checks pass.",
                            );
                        }
                        current_rows(ui, &rows, (&s.id, false));
                        if complete && !challenge {
                            egui::CollapsingHeader::new("Why this repair works")
                                .id_salt(("package-why", &s.id))
                                .show(ui, |ui| {
                                    ui.label(&s.explanation);
                                });
                        }
                    }
                });
                self.metrics.feedback = Some(result.response.rect);
                self.metrics.completion = complete;
                if complete {
                    self.next_unit_lab(ui, i);
                }
            }
        }
        if let Some(notice) = &self.notice {
            ui.colored_label(palette(ui).error, notice);
        }
    }
    pub(super) fn debian_packages_help(&mut self, ui: &mut egui::Ui, i: usize) {
        self.metrics.help_visible = true;
        let blocked = self.reset_pending
            || self.settings_open
            || self.theme_preview
            || self.data_tools.modal_open();
        let s = &self.scenarios[i];
        let a = self.progress.attempts.get_mut(&s.id).unwrap();
        ui.add_enabled_ui(!blocked, |ui| {
            if !a.challenge {
                if a.hints > 0 {
                    ui.label(&s.hints[a.hints - 1]);
                }
                if ui
                    .add_enabled(
                        a.hints < s.hints.len(),
                        button(if a.hints == 0 {
                            "Show a hint"
                        } else {
                            "Another hint"
                        }),
                    )
                    .clicked()
                {
                    a.hints += 1;
                    self.dirty = true;
                }
                egui::CollapsingHeader::new("Optional walkthrough")
                    .id_salt(("package-guide", &s.id))
                    .show(ui, |ui| {
                        for g in &s.guide {
                            ui.label(RichText::new(&g.title).strong());
                            ui.label(&g.why);
                            ui.label(&g.action);
                            for c in &g.commands {
                                ui.add(
                                    egui::Label::new(RichText::new(&c.command).monospace()).wrap(),
                                );
                                ui.label(&c.explanation);
                                if c.simulator_only {
                                    ui.label(
                                        RichText::new(
                                            "Simulator-only helper · not a Linux command",
                                        )
                                        .small()
                                        .color(palette(ui).accent),
                                    );
                                }
                                if ui.add(button("Insert command")).clicked() {
                                    let t = a
                                        .workspace
                                        .as_mut()
                                        .unwrap()
                                        .debian_packages_terminal
                                        .get_or_insert_with(Session::default);
                                    if c.command.len() <= terminal::MAX_COMMAND_BYTES {
                                        t.command = c.command.clone();
                                        self.dirty = true;
                                        self.focus_input = true;
                                    }
                                }
                            }
                        }
                    });
            }
            egui::CollapsingHeader::new("Supported commands, limits & sources")
                .id_salt(("package-reference", &s.id))
                .show(ui, |ui| {
                    ui.label(terminal::HELP);
                    ui.label(&s.simplifications);
                    for source in &s.sources {
                        ui.label(source);
                    }
                });
        });
    }
}
fn current_rows(ui: &mut egui::Ui, rows: &[(bool, String)], scope: (&str, bool)) {
    for (_, message) in rows.iter().filter(|r| !r.0).take(2) {
        ui.colored_label(palette(ui).error, message);
    }
    egui::CollapsingHeader::new("Current checks")
        .id_salt(("package-checks", scope))
        .show(ui, |ui| {
            for (pass, message) in rows {
                ui.label(format!(
                    "{} · {message}",
                    if *pass { "Pass" } else { "Needs work" }
                ));
            }
        });
}
fn displayed(s: &str, command: bool) -> Cow<'_, str> {
    if !s
        .chars()
        .any(|c| c.is_control() && (command || !matches!(c, '\n' | '\t')))
    {
        return Cow::Borrowed(s);
    }
    let mut out = String::new();
    for c in s.chars() {
        if c.is_control() && (command || !matches!(c, '\n' | '\t')) {
            out.extend(c.escape_default());
        } else {
            out.push(c);
        }
    }
    Cow::Owned(out)
}
fn terminal_panel(
    ui: &mut egui::Ui,
    scope: (&str, bool),
    state: &Simulation,
    t: &mut Session,
    metrics: &mut Metrics,
    focus: &mut bool,
) -> InputEffect {
    let mut effect = InputEffect::default();
    if !t.valid_for(state) {
        ui.colored_label(
            palette(ui).error,
            "Package terminal context or retained input exceeds its bounds; previous work retained",
        );
        return effect;
    }
    ui.label(RichText::new("Virtual Debian package terminal").strong());
    ui.label(
        RichText::new(format!(
            "Virtual data · {} package records · Cached index: {} · Supplied publication: {}",
            state.runtime().records.len(),
            state.runtime().cached.as_deref().unwrap_or("none"),
            state.catalog().published
        ))
        .small()
        .color(palette(ui).muted),
    );
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 7. + 2. * f32::from(controls::PAD_Y);
    let output = egui::Frame::new()
        .fill(palette(ui).input)
        .inner_margin(controls::PAD_X)
        .corner_radius(6.)
        .show(ui, |ui| {
            let diagnostic_width =
                (ui.available_width() - ui.spacing().scroll.allocated_width()).max(1.);
            ui.set_min_width(diagnostic_width);
            egui::ScrollArea::both()
                .id_salt(("package-output", scope))
                .min_scrolled_height(height)
                .max_height(height)
                .auto_shrink([false, false])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    if t.entries.is_empty() {
                        ui.label(
                            "Inspect installed records with dpkg-query -W. No host command is run.",
                        );
                    }
                    for e in &t.entries {
                        let command = displayed(&e.command, true);
                        let stdout = displayed(&e.stdout, false);
                        let stderr = displayed(&e.stderr, false);
                        if matches!(command, Cow::Owned(_))
                            || matches!(stdout, Cow::Owned(_))
                            || matches!(stderr, Cow::Owned(_))
                        {
                            ui.label(RichText::new("Control characters shown as escapes").small());
                        }
                        ui.add(
                            egui::Label::new(RichText::new(format!("$ {command}")).monospace())
                                .extend()
                                .selectable(true),
                        );
                        if !stdout.is_empty() {
                            ui.add(
                                egui::Label::new(RichText::new(stdout.as_ref()).monospace())
                                    .extend()
                                    .selectable(true),
                            );
                        }
                        if !stderr.is_empty() {
                            ui.allocate_ui_with_layout(
                                egui::vec2(diagnostic_width, 0.),
                                egui::Layout::top_down(egui::Align::Min),
                                |ui| {
                                    ui.set_width(diagnostic_width);
                                    let text = ui.add(
                                        egui::Label::new(
                                            RichText::new(stderr.as_ref()).color(palette(ui).error),
                                        )
                                        .wrap()
                                        .selectable(true),
                                    );
                                    metrics
                                        .package_diagnostics
                                        .push((text.rect, diagnostic_width));
                                },
                            );
                        }
                        ui.label(
                            RichText::new(format!(
                                "Exit status {}{}",
                                e.status,
                                if e.diagnostic.is_some() {
                                    " · Simulator diagnostic; command retained for correction"
                                } else {
                                    ""
                                }
                            ))
                            .small(),
                        );
                    }
                });
        });
    metrics.output = Some(output.response.rect);
    let id = egui::Id::new(("package-command", scope));
    let had_focus = ui.memory(|m| m.has_focus(id));
    let run_key = ui.is_enabled()
        && had_focus
        && ui.input_mut(|i| {
            i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter)
        });
    let available = ui.available_width();
    let button_width = controls::button_width(ui, "Run command");
    let minimum = ui.text_style_height(&egui::TextStyle::Body) * 8.;
    let field_width = if available >= minimum + button_width + controls::GAP {
        available - button_width - controls::GAP
    } else {
        available
    };
    let old = t.command.clone();
    ui.horizontal_wrapped(|ui| {
        let field = ui.add(
            controls::singleline(ui, &mut t.command)
                .id(id)
                .desired_width((field_width - 2. * f32::from(controls::PAD_X)).max(0.))
                .char_limit(terminal::MAX_COMMAND_BYTES)
                .hint_text("dpkg-query … · Enter to run"),
        );
        name_control(
            &field,
            if scope.1 {
                "Ungraded example Debian package command"
            } else {
                "Assignment Debian package command"
            },
        );
        metrics.package_command = Some((field.rect, id, field.enabled()));
        metrics.terminal = Some(field.rect);
        metrics.inputs.push(field.rect);
        metrics.focus = ui.memory(|m| m.focused());
        if field.changed() {
            if t.command.len() > terminal::MAX_COMMAND_BYTES {
                t.command = old;
                effect.error =
                    Some("Command exceeds2048 UTF8 bytes; previous draft retained".into());
            } else {
                effect.dirty = true;
            }
        }
        if *focus && ui.is_enabled() && !ui.input(|i| i.pointer.any_down()) {
            field.request_focus();
            *focus = false;
        }
        let r = ui.add_enabled(!t.command.trim().is_empty(), button("Run command"));
        metrics.package_run = Some((r.rect, r.enabled()));
        effect.run = ui.is_enabled()
            && effect.error.is_none()
            && !t.command.trim().is_empty()
            && (r.clicked() || run_key);
        if effect.run {
            field.request_focus();
        }
    });
    let before = t.selected_path.clone();
    let inspector = egui::CollapsingHeader::new("Inspect actual virtual files · read-only").id_salt(("package-files",scope)).show(ui,|ui|{
        let paths=match session::known_paths(state){Ok(v)=>v,Err(e)=>{ui.colored_label(palette(ui).error,e);return;}};
        let width=ui.available_width();
        controls::cell(ui,width,|ui|{
            let r=egui::ComboBox::from_id_salt(("package-file",scope)).width(width).truncate().selected_text(t.selected_path.as_deref().unwrap_or("Choose a supplied virtual path")).show_ui(ui,|ui|{
                for p in &paths{let option=ui.selectable_value(&mut t.selected_path,Some(p.clone()),p);metrics.package_path_items.push((p.clone(),option.rect));}metrics.package_path_popup=Some(controls::popup_content_clip(ui));
            });name_control(&r.response,"Read-only virtual package file");metrics.package_path=Some(r.response.rect);
        });
        if let Some(p)=&t.selected_path{
            ui.label(RichText::new(p).monospace());
            match session::current_file(state,p){
                Ok(Some(text))=>{ui.label("Present in the current virtual file map. Recorded ownership is a separate dpkg-query -S investigation.");
                    let text=displayed(text,false);if matches!(text,Cow::Owned(_)){ui.label("Control characters shown as escapes");}
                    egui::ScrollArea::both().id_salt(("package-file-content",scope,p)).max_height(ui.text_style_height(&egui::TextStyle::Monospace)*7.).show(ui,|ui|{ui.add(egui::Label::new(RichText::new(text.as_ref()).monospace()).extend().selectable(true));});
                }
                Ok(None)=>{ui.label("Absent from the current virtual file map. An installed database can still record its owner.");}
                Err(e)=>{ui.colored_label(palette(ui).error,e);}
            }
        }
    });
    metrics.package_inspector = Some(inspector.header_response.rect);
    effect.dirty |= before != t.selected_path;
    effect
}
