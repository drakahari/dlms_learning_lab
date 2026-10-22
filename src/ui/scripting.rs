use super::*;
pub(super) struct Job {
    id: String,
    source: String,
    cases: Vec<shell::Case>,
    index: usize,
    vm: Option<shell::Vm>,
    results: Vec<shell::Outcome>,
}
impl Job {
    fn new(id: String, source: String, cases: Vec<shell::Case>) -> Self {
        Self {
            id,
            source,
            cases,
            index: 0,
            vm: None,
            results: Vec::new(),
        }
    }
}
impl LabApp {
    pub(super) fn advance_scripts(&mut self, ctx: &egui::Context) {
        let Some(job) = &mut self.script_job else {
            return;
        };
        if job.vm.is_none() {
            match shell::Vm::new(&job.source, &job.cases[job.index]) {
                Ok(vm) => job.vm = Some(vm),
                Err(_) => {
                    job.results
                        .push(shell::run(&job.source, &job.cases[job.index]));
                    job.index += 1;
                }
            }
        }
        if let Some(vm) = &mut job.vm {
            vm.tick(16);
            if vm.done() {
                job.results.push(job.vm.take().unwrap().outcome);
                job.index += 1;
            }
        }
        if job.index == job.cases.len() {
            if let Some(a) = self.progress.attempts.get_mut(&job.id)
                && let engine::State::Script { source, run } = &mut a.state
                && *source == job.source
            {
                *run = Some(shell::Record {
                    source: job.source.clone(),
                    results: std::mem::take(&mut job.results),
                });
                a.workspace.as_mut().unwrap().checked = true;
                self.checked = true;
                self.dirty = true;
            }
            self.script_job = None;
        }
        ctx.request_repaint();
    }
    pub(super) fn scripting(&mut self, ui: &mut egui::Ui, i: usize) {
        let s = self.scenarios[i].clone();
        let engine::Goal::Script { cases } = &s.goal else {
            return;
        };
        if !s.prerequisites.is_empty() {
            let prereqs = s
                .prerequisites
                .iter()
                .filter_map(|id| self.scenarios.iter().find(|s| &s.id == id))
                .map(|s| s.title.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            ui.label(
                RichText::new(format!("Builds on: {prereqs}"))
                    .small()
                    .color(palette(ui).muted),
            );
        }
        let mut selected = self.script_case.min(cases.len() - 1);
        ui.horizontal_wrapped(|ui| {
            ui.label("Inspect case");
            egui::ComboBox::from_id_salt("script-case")
                .width(230.)
                .selected_text(&cases[selected].name)
                .show_ui(ui, |ui| {
                    for (n, c) in cases.iter().enumerate() {
                        ui.selectable_value(&mut selected, n, &c.name);
                    }
                });
            ui.label(
                RichText::new(format!(
                    "Run tests all {} cases in fresh virtual workspaces",
                    cases.len()
                ))
                .small()
                .color(palette(ui).muted),
            );
        });
        self.script_case = selected;
        let running = self.script_job.is_some();
        let mut run_clicked = false;
        let mut stop_clicked = false;
        if ui.available_width() >= 850. {
            let width = ui.available_width();
            let left = width * 0.58;
            let workspace_height =
                (ui.ctx().content_rect().height() * 0.32).clamp(180., 320.) + 110.;
            ui.horizontal_top(|ui| {
                ui.allocate_ui_with_layout(
                    egui::vec2(left, workspace_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        ui.set_width(left);
                        self.script_editor(ui, i);
                        let (r, st) = self.script_actions(ui, running);
                        run_clicked = r;
                        stop_clicked = st;
                    },
                );
                ui.allocate_ui_with_layout(
                    egui::vec2((width - left - 12.).max(200.), workspace_height),
                    egui::Layout::top_down(egui::Align::Min),
                    |ui| {
                        self.script_output(ui, i, selected);
                    },
                );
            });
        } else {
            self.script_editor(ui, i);
            let (r, st) = self.script_actions(ui, running);
            run_clicked = r;
            stop_clicked = st;
            self.script_output(ui, i, selected);
        }
        if run_clicked {
            let engine::State::Script { source, run } =
                &mut self.progress.attempts.get_mut(&s.id).unwrap().state
            else {
                return;
            };
            *run = None;
            self.script_job = Some(Job::new(s.id.clone(), source.clone(), cases.clone()));
            self.notice = None;
            self.checked = false;
            self.dirty = true;
            ui.ctx().request_repaint();
        }
        if stop_clicked {
            self.script_job = None;
            self.notice =
                Some("Stopped. No current success is retained; your script stays saved.".into());
            self.checked = false;
            self.dirty = true;
        }
        if let Some(job) = &self.script_job {
            ui.label(format!(
                "Running case {} of {} · bounded in-process simulation",
                job.index + 1,
                job.cases.len()
            ));
        } else if let Some(notice) = &self.notice {
            ui.label(notice);
        }
        let a = &self.progress.attempts[&s.id];
        if let engine::State::Script { source, run } = &a.state
            && run.is_some()
        {
            let checks = shell::checks(source, run, cases);
            let complete = checks.iter().all(|c| c.0);
            self.metrics.completion = complete;
            ui.label(
                RichText::new(if complete {
                    "Objective complete · Every behavior case passes."
                } else {
                    "Needs another edit · Inspect the failing cases, then retry."
                })
                .color(if complete {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            for (pass, message) in checks {
                ui.add(
                    egui::Label::new(RichText::new(message).small().color(if pass {
                        palette(ui).success
                    } else {
                        palette(ui).error
                    }))
                    .wrap(),
                );
            }
            if complete
                && i + 1 < self.scenarios.len()
                && ui
                    .link(format!("Next lab: {}", self.scenarios[i + 1].title))
                    .clicked()
            {
                self.launch(i + 1);
            }
        }
        let c = &cases[selected];
        egui::CollapsingHeader::new("Case inputs, expected behavior & virtual files").id_salt("script-fixture").show(ui,|ui|{ui.label(format!("Arguments: {:?}",c.args));ui.label(format!("Standard input: {:?}",c.stdin));for(k,v)in &c.env{ui.label(format!("{k} = {v:?}"));}ui.label(format!("Expected exit: {} · stdout: {:?} · stderr: {:?}",c.status,c.stdout,c.stderr));for(p,t)in &c.files{ui.label(RichText::new(format!("Input {p}\n{t}")).monospace());}if let engine::State::Script{run:Some(r),..}=&self.progress.attempts[&s.id].state && let Some(result)=r.results.get(selected){for(p,t)in &result.files{ui.label(RichText::new(format!("After run {p}\n{t}")).monospace());}}ui.label("Only the declared virtual files exist. Every Run starts fresh; editor work is retained.");});
    }
    fn script_editor(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.label("Script · portable shell subset");
        let id = egui::Id::new(("script-editor", i));
        let a = self
            .progress
            .attempts
            .get_mut(&self.scenarios[i].id)
            .unwrap();
        let engine::State::Script { source, run } = &mut a.state else {
            return;
        };
        let height = (ui.ctx().content_rect().height() * 0.32).clamp(180., 320.);
        let area = egui::ScrollArea::both()
            .id_salt(("source-scroll", i))
            .max_height(height)
            .show(ui, |ui| {
                ui.add_sized(
                    [ui.available_width(), height],
                    egui::TextEdit::multiline(source)
                        .id(id)
                        .code_editor()
                        .min_size(egui::vec2(ui.available_width(), height - 8.))
                        .desired_width(f32::INFINITY)
                        .char_limit(shell::MAX_SOURCE),
                )
            });
        self.metrics.editor_viewport = Some(area.inner_rect);
        let response = area.inner;
        self.metrics.inputs.push(response.rect);
        self.metrics.focus = Some(id);
        if self.focus_input {
            response.request_focus();
            self.focus_input = false;
        }
        if response.changed() {
            let oversized = source.len() > shell::MAX_SOURCE;
            if oversized {
                let mut end = shell::MAX_SOURCE;
                while !source.is_char_boundary(end) {
                    end -= 1;
                }
                source.truncate(end);
            }
            *run = None;
            self.script_job = None;
            self.checked = false;
            a.workspace.as_mut().unwrap().checked = false;
            self.notice = if oversized {
                Some("Script is limited to 16 KiB. Excess pasted text was not retained.".into())
            } else {
                None
            };
            self.dirty = true;
        }
        ui.label(
            RichText::new("Tab indents here · Ctrl+Enter runs · Esc releases editor focus")
                .small()
                .color(palette(ui).muted),
        );
        if response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            response.surrender_focus();
        }
    }
    fn script_actions(&mut self, ui: &mut egui::Ui, running: bool) -> (bool, bool) {
        let mut run = false;
        let mut stop = false;
        ui.horizontal_wrapped(|ui| {
            let r = ui.add_enabled(!running, primary(ui, "Run script"));
            self.metrics.check = Some(r.rect);
            run = r.clicked()
                || (!running
                    && ui.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Enter)));
            let s = ui.add_enabled(
                running,
                button(if running { "Stop" } else { "Stop (idle)" }),
            );
            self.metrics.stop = Some(s.rect);
            stop = s.clicked();
        });
        (run, stop)
    }
    fn script_output(&self, ui: &mut egui::Ui, i: usize, case: usize) {
        ui.label("Run output · selected case");
        let engine::State::Script { run, .. } =
            &self.progress.attempts[&self.scenarios[i].id].state
        else {
            return;
        };
        let r = run.as_ref().and_then(|r| r.results.get(case));
        egui::Frame::new().fill(palette(ui).input).stroke(egui::Stroke::new(1_f32,palette(ui).border)).inner_margin(10.).show(ui,|ui|{ui.set_min_width(ui.available_width().min(180.));egui::ScrollArea::both().id_salt(("script-output",i,case)).max_height(280.).show(ui,|ui|{if let Some(r)=r{ui.label(format!("Exit status: {}",r.status));ui.label(RichText::new("Standard output").small().color(palette(ui).muted));ui.label(RichText::new(if r.stdout.is_empty(){"(empty)"}else{&r.stdout}).monospace());if !r.stderr.is_empty(){ui.label(RichText::new("Standard error").small().color(palette(ui).error));ui.label(RichText::new(&r.stderr).monospace());}
if let Some(e)=&r.error{ui.add(egui::Label::new(RichText::new(format!("Simulator diagnostic: {e}")).color(palette(ui).error)).wrap());}}else{ui.label("Run the script to see output and feedback. Editing clears the previous result.");}});});
    }
    pub(super) fn appearance_preview(&mut self, ctx: &egui::Context) {
        if self.theme_preview {
            let mut open = true;
            egui::Window::new("Native theme preview").open(&mut open).default_width(500.).resizable(true).show(ctx,|ui|{
                ui.label(format!("{} · {}",theme::find(&self.theme_id).group,theme::find(&self.theme_id).name));
                ui.label("Selection previews immediately and is saved locally. Your desktop theme stays unchanged.");
                egui::ScrollArea::vertical().max_height(ctx.content_rect().height()*0.65).show(ui,|ui|{
                    card(ui).show(ui,|ui|{
                        ui.label("Readable workspace text");
                        ui.label(RichText::new("Secondary instructions").color(palette(ui).muted));
                        let mut text="Select this editor text".to_string();
                        let response=ui.add(egui::TextEdit::singleline(&mut text).id(egui::Id::new("theme-preview-text")));
                        if self.smoke {response.request_focus();if let Some(mut state)=egui::text_edit::TextEditState::load(ctx,response.id){state.cursor.set_char_range(Some(egui::text::CCursorRange::two(egui::text::CCursor::new(0),egui::text::CCursor::new(11))));state.store(ctx,response.id);}}
                        ui.horizontal_wrapped(|ui|{
                            ui.add(primary(ui,"Primary action"));
                            ui.add_enabled(false,button("Unavailable (disabled)"));
                            let _=ui.selectable_label(true,"Selected item");
                        });
                        ui.label(RichText::new("Pass · Configuration valid").color(palette(ui).success));
                        ui.label(RichText::new("Needs correction · Retry the answer").color(palette(ui).error));
                        egui::Frame::new().fill(palette(ui).input).inner_margin(10.).show(ui,|ui|{ui.label(RichText::new("Simulated terminal\nlab # printf '%s\\n' 'ready'\nready").monospace());});
                        ui.label("Tab moves focus; the focused control has a contrasting outline.");
                    });
                    egui::CollapsingHeader::new("Sources & attribution").show(ui,|ui|{
                        ui.label("Five stock DLMS palettes, all 22 official Omarchy v4.0.4 palettes, and the accepted Learning Lab Navy. DLMS and upstream Omarchy Ethereal remain distinct entries.");
                        ui.label("Omarchy revision c668141e9c42b13c80c9ca4ea108e11708c5e8a5 · Native role/contrast adaptations.");
                        ui.label(include_str!("../../assets/themes/LICENSE.dlms"));
                        ui.label(include_str!("../../assets/themes/LICENSE.omarchy"));
                    });
                });
            });
            self.theme_preview = open;
        }
        if let Some(e) = &self.preferences_error {
            egui::TopBottomPanel::bottom("appearance-error").show(ctx, |ui| {
                ui.label(format!("Appearance could not be saved: {e}"));
            });
        }
    }
}
