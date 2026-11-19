//! Native time-zone terminal: real parsed investigation with separate practice.
use super::*;
use dlms_learning_lab::{timezone_workspace as work, timezones};
#[derive(Default)]
pub(super) struct Metrics {
    pub summary: Option<String>,
    pub disclosure_ids: Vec<egui::Id>,
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
}
impl Metrics {
    fn record(&mut self, ui: &egui::Ui, key: &str, r: &egui::Response) {
        self.controls.push((key.into(), r.rect, ui.clip_rect()));
    }
}
fn code(ui: &mut egui::Ui, text: &str) {
    lesson_presentation::monospace(ui, text, true);
}
fn guidance(ui: &mut egui::Ui, s: &pack::Scenario, n: usize, practice: bool) {
    if practice {
        lesson_presentation::practice_step(ui, s, n);
    }
}
fn provided(ui: &mut egui::Ui, t: &work::Task, metrics: &mut Metrics) {
    ui.strong("Supplied UTC instants (first sample is virtual now)");
    let body = egui::TextStyle::Body.resolve(ui.style());
    let mono = egui::TextStyle::Monospace.resolve(ui.style());
    let color = palette(ui).text;
    let measure = |ui: &mut egui::Ui, text: &str, font: &egui::FontId| {
        ui.fonts_mut(|f| f.layout_no_wrap(text.into(), font.clone(), color).size().x)
    };
    let mut widths = [
        measure(ui, "Sample", &body),
        measure(ui, "UTC reading", &body),
        measure(ui, "Unix seconds", &body),
    ];
    for moment in &t.facts.moments {
        widths[0] = widths[0].max(measure(ui, &moment.label, &body));
        widths[1] = widths[1].max(measure(
            ui,
            &timezones::Zone::Utc.at(moment.epoch).unwrap().civil.iso(),
            &mono,
        ));
        widths[2] = widths[2].max(measure(ui, &format!("@{}", moment.epoch), &mono));
    }
    let gap = ui.spacing().item_spacing.x * 2.;
    for width in &mut widths {
        *width += ui.spacing().item_spacing.x;
    }
    if ui.available_width() >= widths.iter().sum::<f32>() + gap * 2. {
        egui::Grid::new(("timezone-moments", &t.id))
            .spacing(egui::vec2(gap, ui.spacing().item_spacing.y))
            .show(ui, |ui| {
                for (n, title) in ["Sample", "UTC reading", "Unix seconds"].iter().enumerate() {
                    controls::cell(ui, widths[n], |ui| ui.label(*title));
                }
                ui.end_row();
                for (n, moment) in t.facts.moments.iter().enumerate() {
                    controls::cell(ui, widths[0], |ui| ui.label(&moment.label));
                    let r = controls::cell(ui, widths[1], |ui| {
                        lesson_presentation::monospace(
                            ui,
                            &timezones::Zone::Utc.at(moment.epoch).unwrap().civil.iso(),
                            false,
                        )
                    })
                    .inner;
                    metrics.record(ui, &format!("moment-{n}-utc"), &r);
                    let r = controls::cell(ui, widths[2], |ui| {
                        lesson_presentation::monospace(ui, &format!("@{}", moment.epoch), false)
                    })
                    .inner;
                    metrics.record(ui, &format!("moment-{n}-epoch"), &r);
                    ui.end_row();
                }
            });
    } else {
        for (n, moment) in t.facts.moments.iter().enumerate() {
            ui.label(&moment.label);
            let r = lesson_presentation::monospace(
                ui,
                &format!(
                    "{} UTC",
                    timezones::Zone::Utc.at(moment.epoch).unwrap().civil.iso()
                ),
                true,
            );
            metrics.record(ui, &format!("moment-{n}-utc"), &r);
            let r = lesson_presentation::monospace(
                ui,
                &format!("Unix seconds: @{}", moment.epoch),
                true,
            );
            metrics.record(ui, &format!("moment-{n}-epoch"), &r);
            ui.add_space(ui.spacing().item_spacing.y);
        }
    }
}
fn commands(t: &work::Task) -> Vec<&str> {
    let mut seen = std::collections::BTreeSet::new();
    t.reference
        .iter()
        .map(|c| c.text.as_str())
        .filter(|s| seen.insert(*s))
        .collect()
}
fn history(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, m: &mut Metrics) {
    let area = ui.scope(|ui| {
        ui.strong("Actual terminal results");
        let last = d.last_output(t);
        match &last {
            Ok(None) => {
                ui.label("Nothing has run in this draft yet.");
            }
            Ok(Some(Ok(out))) => {
                ui.label(format!("Last exit status: {}", out.status));
            }
            Ok(Some(Err(e))) => {
                let label = match e.kind {
                    dlms_learning_lab::boot_entry::ErrorKind::Syntax => "Command syntax",
                    dlms_learning_lab::boot_entry::ErrorKind::Unsupported => {
                        "Outside the supported subset"
                    }
                    dlms_learning_lab::boot_entry::ErrorKind::Limit => "Simulator limit",
                    _ => "Command could not run",
                };
                lesson_presentation::rich(
                    ui,
                    RichText::new(format!("{label}: {e}")).color(palette(ui).error),
                );
            }
            Err(e) => {
                lesson_presentation::rich(ui, RichText::new(e).color(palette(ui).error));
            }
        }
        let height = ui.text_style_height(&egui::TextStyle::Monospace) * 12.;
        controls::nested_scroll(
            ui,
            egui::ScrollArea::both()
                .id_salt("timezone-output")
                .stick_to_bottom(true)
                .max_height(height)
                .auto_shrink([false, true]),
            |ui| {
                let mut world = match t.initial.world() {
                    Ok(w) => w,
                    Err(e) => {
                        ui.label(e);
                        return;
                    }
                };
                let start = d.history.len().saturating_sub(8);
                for (n, c) in d.history.iter().enumerate() {
                    match world.command(&c.text) {
                        Ok(out) if n >= start => {
                            code(ui, &format!("$ {}", c.text));
                            if !out.stdout.is_empty() {
                                code(ui, &out.stdout);
                            }
                            if !out.stderr.is_empty() {
                                lesson_presentation::rich(
                                    ui,
                                    RichText::new(out.stderr).color(palette(ui).error),
                                );
                            }
                            if out.status != 0 {
                                code(ui, &format!("[exit status {}]", out.status));
                            }
                            ui.add_space(ui.spacing().item_spacing.y);
                        }
                        Ok(_) => {}
                        Err(e) => {
                            ui.label(format!("Cannot replay saved terminal results: {e}"));
                            break;
                        }
                    }
                }
                if let Some(r) = &d.last_run
                    && !r.appended
                {
                    code(ui, &format!("$ {}", r.command.text));
                }
            },
        );
    });
    m.record(ui, "last-result", &area.response);
}
fn terminal(
    ui: &mut egui::Ui,
    t: &work::Task,
    d: &mut work::Draft,
    practice: bool,
    active: bool,
    m: &mut Metrics,
    notice: &mut Option<String>,
) {
    let label = ui.strong("Simulated terminal");
    if practice {
        let commands = commands(t);
        let id = ui.make_persistent_id("timezone-worked-command");
        let mut selected = ui
            .ctx()
            .data(|data| data.get_temp::<usize>(id))
            .unwrap_or(0)
            .min(commands.len().saturating_sub(1));
        let menu = controls::dropdown(ui, "timezone-command-menu", ui.available_width())
            .selected_text(
                commands
                    .get(selected)
                    .copied()
                    .unwrap_or("No worked command"),
            )
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (n, text) in commands.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, n, *text);
                    m.record(ui, &format!("worked-{n}"), &r);
                }
                let clip = controls::popup_content_clip(ui);
                for r in &mut m.controls {
                    if r.0.starts_with("worked-") {
                        r.2 = clip;
                    }
                }
            });
        m.record(ui, "worked-menu", &menu.response);
        ui.ctx().data_mut(|data| data.insert_temp(id, selected));
        let r = ui.add_enabled(active, button("Insert command"));
        m.record(ui, "insert", &r);
        if r.clicked()
            && let Some(text) = commands.get(selected)
        {
            d.input = (*text).into();
        }
        ui.label("Insert fills the input. Run command executes it in this simulator.");
    }
    let mut input = d.input.clone();
    let (enter, clicked) =
        if ui.available_width() > ui.text_style_height(&egui::TextStyle::Body) * 24. {
            ui.horizontal(|ui| {
                let width = controls::button_width(ui, "Run command");
                let field = ui
                    .add_enabled(
                        active,
                        controls::singleline_fitting(
                            ui,
                            &mut input,
                            (ui.available_width() - width - ui.spacing().item_spacing.x).max(80.),
                        )
                        .hint_text("Command from Supported commands")
                        .char_limit(1024),
                    )
                    .labelled_by(label.id);
                m.record(ui, "command", &field);
                let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                let r = ui
                    .add_enabled_ui(active, |ui| {
                        ui.add_sized([width, controls::height(ui)], button("Run command"))
                    })
                    .inner;
                m.record(ui, "run", &r);
                (enter, r.clicked())
            })
            .inner
        } else {
            let field = ui
                .add_enabled(
                    active,
                    controls::singleline_fitting(ui, &mut input, ui.available_width())
                        .hint_text("Command from Supported commands")
                        .char_limit(1024),
                )
                .labelled_by(label.id);
            m.record(ui, "command", &field);
            let enter = field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let r = ui.add_enabled(active, button("Run command"));
            m.record(ui, "run", &r);
            (enter, r.clicked())
        };
    if input.len() <= 1024 && !input.contains('\0') {
        d.input = input;
    } else {
        *notice = Some("Keep one command within1,024bytes with no zero byte.".into());
    }
    if active && (clicked || enter) {
        *notice = d.run(t).err();
    }
    history(ui, t, d, m);
}
fn feedback(ui: &mut egui::Ui, t: &work::Task, d: &work::Draft, practice: bool, m: &mut Metrics) {
    if !d.attempted {
        ui.label("Not checked");
        return;
    }
    let rows = match d.checks(t) {
        Ok(rows) => rows,
        Err(e) => {
            ui.label(e);
            return;
        }
    };
    let failed = rows.iter().filter(|r| !r.correct).count();
    let summary = if failed == 0 {
        if practice {
            "Example meets the task".into()
        } else {
            "Assignment complete".into()
        }
    } else {
        format!(
            "Needs attention · {failed} requirement{} fail{}",
            if failed == 1 { "" } else { "s" },
            if failed == 1 { "s" } else { "" }
        )
    };
    m.summary = Some(summary.clone());
    let r = lesson_presentation::rich(
        ui,
        RichText::new(summary).color(if failed == 0 {
            palette(ui).success
        } else {
            palette(ui).error
        }),
    );
    m.record(ui, "result", &r);
    let draw = |ui: &mut egui::Ui, n: usize, row: &work::Check, m: &mut Metrics| {
        let area = ui.scope(|ui| {
            lesson_presentation::rich(
                ui,
                RichText::new(format!(
                    "{} · {}: {}",
                    if row.correct {
                        "Verified"
                    } else {
                        "Needs attention"
                    },
                    row.label,
                    row.feedback
                ))
                .color(if row.correct {
                    palette(ui).success
                } else {
                    palette(ui).error
                }),
            );
            if !row.correct
                && let (Some(expected), Some(actual)) = (&row.expected, &row.actual)
            {
                let detail = controls::collapsing_header(ui, "Expected versus actual · optional")
                    .id_salt(("timezone-case-comparison", n))
                    .show(ui, |ui| {
                        ui.strong("Expected");
                        code(ui, expected);
                        ui.strong("Actual");
                        code(ui, actual);
                    });
                m.disclosure_ids.push(detail.header_response.id);
            }
        });
        m.record(ui, &format!("feedback-{n}"), &area.response);
    };
    for (n, row) in rows.iter().enumerate().filter(|(_, r)| !r.correct) {
        draw(ui, n, row, m);
    }
    let passed = rows.len() - failed;
    if passed > 0 {
        let r = controls::collapsing_header(ui, &format!("Show {passed} passing checks")).show(
            ui,
            |ui| {
                for (n, row) in rows.iter().enumerate().filter(|(_, r)| r.correct) {
                    draw(ui, n, row, m);
                }
            },
        );
        m.record(ui, "passing-checks", &r.header_response);
    }
}
impl LabApp {
    pub(super) fn timezone_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if previous != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let attempt = &self.progress.attempts[&source.id];
        let def =
            match work::source(&source).and_then(|d| work::context(attempt, &source).map(|()| d)) {
                Ok(d) => d,
                Err(e) => {
                    self.notice = Some(e);
                    return;
                }
            };
        let engine::State::Timezone { simulation } = &attempt.state else {
            return;
        };
        let before = (**simulation).clone();
        let mut next = before.clone();
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
        self.metrics.timezone = Metrics::default();
        ui.heading(&source.title);
        if !challenge {
            ui.horizontal_wrapped(|ui| {
                for (choice, key, label) in [
                    (teaching::View::Introduction, "introduction", "Introduction"),
                    (teaching::View::Example, "example", "Try example · ungraded"),
                    (teaching::View::Assignment, "assignment", "Assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label).selected(view == choice));
                    self.metrics.timezone.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = choice;
                        self.lesson_scroll_reset = true;
                        self.notice = None;
                    }
                }
            });
            next.view = view;
        }
        ui.push_id((i,view as u8),|ui|{
            if view==teaching::View::Introduction {
                lesson_presentation::concepts(ui,&source,"Builds on terminal and locale foundations; all supplied instants and settings are virtual.",Vec::<(String,String)>::new());
                lesson_presentation::worked(ui,"A different regional example demonstrates the method before independent work.");
                for(n,r)in lesson_presentation::worked_command_rows(ui,&source).iter().enumerate(){self.metrics.timezone.record(ui,&format!("worked-value-{n}"),r);}
                ui.horizontal_wrapped(|ui|{
                    for(choice,key,label)in[(teaching::View::Example,"try","Try this example"),(teaching::View::Assignment,"start","Open full assignment")]{
                        let r=ui.add_enabled(active,button(label));self.metrics.timezone.record(ui,key,&r);if r.clicked(){next.view=choice;self.lesson_scroll_reset=true;}
                    }
                });
            }else{
                let practice=view==teaching::View::Example;if !practice{lesson_presentation::assignment(ui,&source);}
                let(task,draft)=if practice{(&def.example,&mut next.practice)}else{(&def.initial,&mut next.assignment)};
                ui.add_enabled_ui(active,|ui|{
                    provided(ui,task,&mut self.metrics.timezone);ui.separator();
                    let usable=active&&draft.history.len()<work::MAX_HISTORY;
                    if practice&&ui.available_width()>ui.text_style_height(&egui::TextStyle::Body)*48. {
                        ui.columns(2,|columns|{
                            guidance(&mut columns[0],&source,0,true);guidance(&mut columns[0],&source,1,true);
                            terminal(&mut columns[1],task,draft,true,usable,&mut self.metrics.timezone,&mut self.notice);
                        });
                    }else{
                        guidance(ui,&source,0,practice);
                        terminal(ui,task,draft,practice,usable,&mut self.metrics.timezone,&mut self.notice);
                        guidance(ui,&source,1,practice);
                    }
                    ui.separator();guidance(ui,&source,2,practice);
                    let r=ui.add_enabled(active,button("Check work").fill(palette(ui).accent));self.metrics.timezone.record(ui,"check",&r);
                    if r.clicked(){self.notice=draft.check(task).err();}
                    feedback(ui,task,draft,practice,&mut self.metrics.timezone);
                    if draft.history.len()>=work::MAX_HISTORY{ui.label("History reached its96-action limit. Reset only this example or selected attempt to start over.");}
                    if practice{let r=ui.add_enabled(active,button("Reset example only"));self.metrics.timezone.record(ui,"reset-example",&r);if r.clicked(){*draft=work::Draft::new(task);self.notice=None;}}
                    controls::collapsing_header(ui,"Supported commands and help · optional").show(ui,|ui|{
                        lesson_presentation::prose(ui,work::HELP);
                        let attempt=self.progress.attempts.get_mut(&source.id).unwrap();
                        let r=ui.add_enabled(active&&attempt.hints<source.hints.len(),button("Show a hint"));
                        if r.clicked(){attempt.hints+=1;self.dirty=true;}
                        for hint in source.hints.iter().take(attempt.hints){lesson_presentation::prose(ui,hint);}
                        lesson_presentation::prose(ui,&source.simplifications);
                        controls::collapsing_header(ui,"Technical references · optional").show(ui,|ui|{
                            for url in &source.sources {ui.hyperlink_to(url,url);}
                        });
                        controls::collapsing_header(ui,"Worked approach · reveals one method").show(ui,|ui|lesson_presentation::prose(ui,&source.explanation));
                    });
                });
            }
        });
        if next != before {
            self.progress.attempts.get_mut(&source.id).unwrap().state = engine::State::Timezone {
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
