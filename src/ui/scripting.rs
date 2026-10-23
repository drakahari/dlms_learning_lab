use super::*;
fn input_value_preview(value: &str) -> String {
    let end = value.char_indices().nth(80).map_or(value.len(), |(n, _)| n);
    let prefix = format!("{:?}", &value[..end]);
    if end == value.len() {
        prefix
    } else {
        format!(
            "{prefix} … ({} bytes; full value in input details)",
            value.len()
        )
    }
}

pub(super) struct Job {
    id: String,
    source: String,
    cases: Vec<shell::Case>,
    index: usize,
    vm: Option<shell::Vm>,
    results: Vec<shell::Outcome>,
    example: bool,
}
impl Job {
    pub(super) fn example(id: String, source: String, case: shell::Case) -> Self {
        let mut job = Self::new(id, source, vec![case]);
        job.example = true;
        job
    }
    fn new(id: String, source: String, cases: Vec<shell::Case>) -> Self {
        Self {
            id,
            source,
            cases,
            index: 0,
            vm: None,
            results: Vec::new(),
            example: false,
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
            if let Some(a) = self.progress.attempts.get_mut(&job.id) {
                if job.example {
                    if let Some(l) = a
                        .workspace
                        .as_mut()
                        .and_then(|w| w.script_learning.as_mut())
                        && l.example_source == job.source
                    {
                        l.example_result = job.results.pop();
                        l.example_revision = shell::EVALUATOR_REVISION;
                        self.dirty = true;
                    }
                } else if let engine::State::Script { source, run } = &mut a.state
                    && *source == job.source
                {
                    self.script_case = job
                        .results
                        .iter()
                        .zip(&job.cases)
                        .position(|(r, c)| !c.accepts(r))
                        .unwrap_or(0);
                    *run = Some(shell::Record {
                        engine_revision: dlms_learning_lab::shell::EVALUATOR_REVISION,
                        source: job.source.clone(),
                        results: std::mem::take(&mut job.results),
                    });
                    a.workspace.as_mut().unwrap().checked = true;
                    self.checked = true;
                    self.dirty = true;
                }
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
            ui.label("Inspect inputs");
            let picker_width = 230_f32.min(ui.available_width());
            ui.allocate_ui_with_layout(egui::vec2(picker_width, controls::height(ui)), egui::Layout::left_to_right(egui::Align::Center), |ui| {
                let picker=egui::ComboBox::from_id_salt("script-case")
                    .width(picker_width)
                    .truncate()
                    .selected_text(&cases[selected].name)
                    .show_ui(ui, |ui| {
                        for (n, c) in cases.iter().enumerate() {
                            ui.selectable_value(&mut selected, n, &c.name);
                        }
                    });
                name_control(&picker.response,"Inspect script test inputs");
            });
            ui.label(
                RichText::new(format!(
                    "Run script checks {} sets of sample inputs; simulated files reset for each test",
                    cases.len()
                ))
                .small()
                .color(palette(ui).muted),
            );
        });
        self.script_case = selected;
        let c = &cases[selected];
        if !c.env.is_empty() {
            ui.add(
                egui::Label::new(format!(
                    "Supplied by the lab for these inputs: {}",
                    c.env
                        .iter()
                        .map(|(name, value)| format!("{name} = {}", input_value_preview(value)))
                        .collect::<Vec<_>>()
                        .join(" · ")
                ))
                .wrap(),
            );
        }

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
                    "Correct solution · All input sets pass."
                } else {
                    "Needs another edit · Inspect the failing inputs, then retry."
                })
                .color(if complete {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            let same_error = run.as_ref().is_some_and(|r| {
                r.current()
                    && r.results.len() == cases.len()
                    && r.results
                        .first()
                        .and_then(|r| r.error.as_ref())
                        .is_some_and(|first| {
                            r.results.iter().all(|r| r.error.as_ref() == Some(first))
                        })
            });
            if same_error {
                ui.label(format!(
                    "The same diagnostic stopped all {} input sets. Fix it once, then Run again.",
                    cases.len()
                ));
            } else {
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
            }
            if complete {
                self.next_unit_lab(ui, i);
            }
        }
        let c = &cases[selected];
        egui::CollapsingHeader::new("Input details, expected results & virtual files").id_salt("script-fixture").show(ui,|ui|{egui::ScrollArea::vertical().id_salt("script-fixture-scroll").max_height(260.).show(ui,|ui|{ui.label(format!("Arguments: {:?}",c.args));ui.label(format!("Standard input: {:?}",c.stdin));for(k,v)in &c.env{ui.label(format!("{k} = {v:?}"));}ui.label(format!("Expected exit: {} · stdout: {:?} · stderr: {:?}",c.status,c.stdout,c.stderr));for(p,t)in &c.files{ui.label(RichText::new(format!("Input {p}\n{t}")).monospace());}if let engine::State::Script{run:Some(r),..}=&self.progress.attempts[&s.id].state && let Some(result)=r.results.get(selected){for(p,t)in &result.files{ui.label(RichText::new(format!("After run {p}\n{t}")).monospace());}}ui.label("Only the listed simulated files exist. Each test resets these files; your script stays saved.");});});
    }
    fn script_editor(&mut self, ui: &mut egui::Ui, i: usize) {
        ui.label("Script · simulated shell");
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
            .min_scrolled_height(height)
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
        name_control(&response, "Assignment script source");
        self.metrics.inputs.push(response.rect);
        self.metrics.focus = Some(id);
        if self.focus_input && ui.is_enabled() {
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
                    && ui.is_enabled()
                    && self
                        .metrics
                        .focus
                        .is_some_and(|id| ui.memory(|m| m.has_focus(id)))
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
    fn script_output(&mut self, ui: &mut egui::Ui, i: usize, case: usize) {
        ui.label("Run output · selected inputs");
        let engine::State::Script { run, .. } =
            &self.progress.attempts[&self.scenarios[i].id].state
        else {
            return;
        };
        let workspace = self.progress.attempts[&self.scenarios[i].id]
            .workspace
            .as_ref();
        let histories: Vec<_> = workspace
            .into_iter()
            .flat_map(|w| {
                w.script_history
                    .as_ref()
                    .into_iter()
                    .chain(&w.script_history_more)
            })
            .collect();
        let displayed = run.as_ref().or_else(|| histories.last().copied());
        let earlier = displayed.is_some_and(|r| !r.current());
        if earlier {
            ui.label("Earlier saved output · Your script and results are retained. Run the current draft to verify it with this simulator.");
        }
        if !histories.is_empty() {
            let history_panel = egui::CollapsingHeader::new(if histories.len() == 1 {
                "Earlier saved script"
            } else {
                "Earlier saved runs"
            })
            .id_salt(("script-history", i))
            .show(ui, |ui| {
                for (history_index, history) in histories.iter().enumerate().rev() {
                    ui.label(format!(
                        "Saved evaluator revision {} · Your earlier work",
                        history.engine_revision
                    ));
                    ui.add(egui::Label::new(RichText::new(&history.source).monospace()).wrap());
                    if let Some(result) = history.results.get(case) {
                        egui::CollapsingHeader::new("Saved output and virtual files")
                            .id_salt((i, history_index))
                            .show(ui, |ui| {
                                outcome(ui, result);
                                for (name, text) in &result.files {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(format!("{name}\n{text}")).monospace(),
                                        )
                                        .wrap(),
                                    );
                                }
                            });
                    }
                }
            });
            self.metrics.script_history = Some(history_panel.header_response.rect);
        }
        let r = displayed.and_then(|r| r.results.get(case));
        let output = egui::Frame::new().fill(palette(ui).input).stroke(egui::Stroke::new(1_f32,palette(ui).border)).inner_margin(10.).show(ui, |ui| {
            egui::ScrollArea::both().id_salt(("script-output",i,case)).max_height(330.).min_scrolled_height(if r.is_some() { 160. } else { 40. }).auto_shrink([false,r.is_none()]).show(ui, |ui| {
                if let Some(r) = r {
                    outcome(ui, r);
                    let engine::Goal::Script { cases } = &self.scenarios[i].goal else { return; };
                    let c = &cases[case];
                    if !earlier && r.error.is_none() && !c.accepts(r) {
                        ui.label(RichText::new("Behavior differs · Edit your script and Run again.").color(palette(ui).error));
                        if !self.scenarios[i].retry_guidance.is_empty() { ui.label(&self.scenarios[i].retry_guidance); }
                        if c.stdout != r.stdout { if self.scenarios[i].retry_guidance.is_empty() { ui.label(teaching::output_advice(&c.stdout,&r.stdout)); } output_diff(ui,"Standard output", &c.stdout, &r.stdout); }
                        if c.stderr != r.stderr { ui.label("Standard error is the separate channel for problem messages; >&2 sends text there."); output_diff(ui,"Standard error", &c.stderr, &r.stderr); }
                        if c.status != r.status { ui.label(format!("Expected exit status: {} · Actual: {}. A successful task uses 0; an input problem uses the assignment’s stated nonzero status.", c.status, r.status)); }
                        if c.after != r.files { ui.label("The virtual files differ. Check the destination path, filter and whether > replaced or >> appended text. Inspect the files below."); }
                    }
                } else { ui.label("Type your script in the editor, then choose Run script. Output and feedback appear here."); }
            });
        });
        self.metrics.output = Some(output.response.rect);
    }
    pub(super) fn appearance_preview(&mut self, ctx: &egui::Context) {
        if self.settings_open || self.theme_preview {
            let mut open = true;
            egui::Window::new("Settings").open(&mut open).default_width(500.).resizable(true).show(ctx,|ui|{
                ui.horizontal_wrapped(|ui| { ui.selectable_value(&mut self.settings_data,false,"Appearance");ui.selectable_value(&mut self.settings_data,true,"Your data"); });
                if self.settings_data { egui::ScrollArea::vertical().id_salt("data-settings-scroll").max_height(ctx.content_rect().height()*0.65).show(ui,|ui|self.data_settings(ui)); } else {
                ui.heading("Appearance");
                ui.label(RichText::new(format!("Version {} · Build {}",env!("CARGO_PKG_VERSION"),option_env!("DLMS_BUILD_REVISION").unwrap_or("development").chars().take(12).collect::<String>())).small());
                ui.push_id("settings-theme", |ui| { self.theme_dropdown(ui); });
                ui.label(format!("{} · {}",theme::find(&self.theme_id).group,theme::find(&self.theme_id).name));
                ui.label("Selection previews immediately and is saved locally. Your desktop theme stays unchanged.");
                egui::ScrollArea::vertical().max_height(ctx.content_rect().height()*0.65).show(ui,|ui|{
                    egui::CollapsingHeader::new("Theme samples").default_open(self.theme_preview).show(ui, |ui| {
                    ui.label("Examples only · These controls demonstrate appearance and perform no lab actions.");
                    card(ui).show(ui,|ui|{
                        ui.label("Readable workspace text");
                        ui.label(RichText::new("Secondary instructions").color(palette(ui).muted));
                        let mut text="Select this editor text".to_string();
                        let response=ui.add(controls::singleline(ui, &mut text).id(egui::Id::new("theme-preview-text")));
                        name_control(&response,"Theme sample editor — appearance example only");
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
                    });
                    egui::CollapsingHeader::new("Sources & attribution").show(ui,|ui|{
                        ui.label("Five stock DLMS palettes, all 22 official Omarchy v4.0.4 palettes, and the accepted Learning Lab Navy. DLMS and upstream Omarchy Ethereal remain distinct entries.");
                        ui.label("Omarchy revision c668141e9c42b13c80c9ca4ea108e11708c5e8a5 · Native role/contrast adaptations.");
                        ui.label(include_str!("../../assets/themes/LICENSE.dlms"));
                        ui.label(include_str!("../../assets/themes/LICENSE.omarchy"));
                    });
                });
                }
            });
            self.settings_open = open;
            if !open {
                self.theme_preview = false;
            }
        }
        if let Some(e) = &self.preferences_error {
            egui::TopBottomPanel::bottom("appearance-error").show(ctx, |ui| {
                ui.label(format!("Appearance could not be saved: {e}"));
            });
        }
    }
}

fn output_diff(ui: &mut egui::Ui, label: &str, expected: &str, actual: &str) {
    ui.label(RichText::new(format!("{label} · \\n means a line break below")).small());
    ui.add(
        egui::Label::new(
            RichText::new(format!("Expected: {expected:?}\nActual:   {actual:?}")).monospace(),
        )
        .wrap(),
    );
}
pub(super) fn outcome(ui: &mut egui::Ui, r: &shell::Outcome) {
    ui.label(format!("Exit status: {}", r.status));
    ui.label(
        RichText::new("Standard output · printed text")
            .small()
            .color(palette(ui).muted),
    );
    ui.add(
        egui::Label::new(
            RichText::new(if r.stdout.is_empty() {
                "(empty)"
            } else {
                &r.stdout
            })
            .monospace(),
        )
        .wrap(),
    );
    if !r.stderr.is_empty() {
        ui.label(
            RichText::new("Standard error · problem messages")
                .small()
                .color(palette(ui).error),
        );
        ui.add(egui::Label::new(RichText::new(&r.stderr).monospace()).wrap());
    }
    if let Some(e) = &r.error {
        ui.label(RichText::new(teaching::diagnostic_title(e)).color(palette(ui).error));
        ui.label(teaching::diagnostic_advice(e));
        ui.add(egui::Label::new(e).wrap());
        ui.label("Edit the script, then Run again. Your draft stays saved.");
    }
}
