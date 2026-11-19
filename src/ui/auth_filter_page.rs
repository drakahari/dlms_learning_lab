//! Native filter editor and actual line observations over the bounded simulator.
use super::linux_v6_page::Metrics;
use super::*;
use dlms_learning_lab::{
    auth_filter_source as binding,
    auth_filter_task::Task,
    auth_filter_workspace::{self as work, Action},
    log_filter::Disposition,
};
fn apply(s: &mut work::Scope, t: &Task, a: Action, n: &mut Option<String>) {
    if let Err(e) = s.apply(t, &a) {
        *n = Some(e);
    }
}
fn editor(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    let label = ui.strong("Filter editor");
    lesson_presentation::monospace(ui, &t.filter_path, true);
    let mut text = s.editor.clone();
    let font = egui::TextStyle::Monospace.resolve(ui.style());
    let color = ui.visuals().text_color();
    let mut layouter = |ui: &egui::Ui, b: &dyn egui::TextBuffer, _width: f32| {
        ui.fonts_mut(|f| f.layout_no_wrap(b.as_str().to_owned(), font.clone(), color))
    };
    let height =
        ui.text_style_height(&egui::TextStyle::Monospace) * 8. + 2. * f32::from(controls::PAD_Y);
    let size = egui::vec2(
        ui.available_width(),
        height + ui.spacing().scroll.allocated_width(),
    );
    let (outer, r) = controls::fixed_viewport(ui, size, |ui| {
        egui::ScrollArea::both()
            .id_salt(("auth-filter-editor-scroll", &t.id))
            .auto_shrink([false, false])
            .min_scrolled_height(height)
            .max_height(size.y)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut text)
                        .id_salt(("auth-filter-editor", &t.id))
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(8)
                        .char_limit(2048)
                        .margin(egui::Margin::symmetric(controls::PAD_X, controls::PAD_Y))
                        .lock_focus(false)
                        .layouter(&mut layouter),
                )
            })
    });
    m.controls
        .push(("editor-fixed".into(), outer, ui.clip_rect()));
    m.controls
        .push(("editor-viewport".into(), r.inner_rect, ui.clip_rect()));
    m.controls.push((
        "editor".into(),
        r.inner.rect.intersect(r.inner_rect),
        ui.clip_rect(),
    ));
    let response = r.inner.labelled_by(label.id);
    m.fields.push(("editor".into(), response.rect, response.id));
    m.focus.push(("editor".into(), response.id));
    if text != s.editor {
        apply(s, t, Action::Edit { text }, n);
    }
    ui.horizontal_wrapped(|ui| {
        let r = ui.add(button("Save virtual filter"));
        m.record(ui, "save", &r);
        if r.clicked() {
            apply(s, t, Action::Save, n);
        }
        ui.label(if s.editor == s.saved {
            "Saved"
        } else {
            "Unsaved changes · commands still read the saved filter"
        });
    });
}
fn observations(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    ui.strong("Supplied log · actual line results");
    lesson_presentation::monospace(ui, &t.log_path, true);
    let mut selected = s.selected;
    controls::form_row(ui, "Log sample", |ui, width| {
        let r = controls::dropdown(ui, ("auth-filter-sample", &t.id), width)
            .selected_text(&t.cases[selected].label)
            .show_ui(ui, |ui| {
                controls::fit_popup_contents(ui);
                for (i, c) in t.cases.iter().enumerate() {
                    let r = ui.selectable_value(&mut selected, i, &c.label);
                    m.record(ui, &format!("sample:{i}"), &r);
                }
            });
        controls::retain_dropdown_focus(ui, &r.response);
        m.record(ui, "sample", &r.response);
    });
    if selected != s.selected {
        apply(s, t, Action::Select { case: selected }, n);
    }
    let actual = s
        .last_run
        .as_ref()
        .filter(|r| r.case == s.selected && r.filter == s.saved)
        .and_then(|r| t.observe(&r.filter, r.case).ok());
    let heading = ui.label(if actual.is_none() {
        "Unchecked sample · Run command to test its lines"
    } else if s.editor != s.saved {
        "Last saved-file test · editor changes are not saved"
    } else {
        "Actual saved-filter test"
    });
    m.record(ui, "observations-state", &heading);
    // Allocate widths before entering Grid; available_width inside a grid cell
    // is not the enclosing workspace width.
    let gap = ui.spacing().item_spacing.x;
    let status = controls::button_width(ui, "64 · Unchecked").min(ui.available_width() * 0.4);
    let text_width = (ui.available_width() - status - gap).max(1.);
    // Measure both cells before allocating the row. Centering wrapped text in
    // a single-line-height child lets the next stripe paint over its last line.
    let cells = t.cases[s.selected]
        .lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let label = actual
                .as_ref()
                .map(|rows| match rows[i].disposition {
                    Disposition::Matched => "Matched",
                    Disposition::Ignored => "Ignored",
                    Disposition::Missed => "Missed",
                })
                .unwrap_or("Unchecked");
            let layout = |text: egui::WidgetText, width| {
                text.into_galley(
                    ui,
                    Some(egui::TextWrapMode::Wrap),
                    width,
                    egui::TextStyle::Body,
                )
            };
            let left = layout(format!("{} · {label}", i + 1).into(), status);
            let right = layout(RichText::new(line).monospace().into(), text_width);
            let height = left
                .size()
                .y
                .max(right.size().y)
                .max(controls::table_height(ui));
            (left, right, height)
        })
        .collect::<Vec<_>>();
    // Allocate each pair from this frame's measured text, not Grid's previous
    // row-height cache. A narrower/shorter row must not paint over its predecessor.
    let r = ui.vertical(|ui| {
        for (i, (left, right, height)) in cells.into_iter().enumerate() {
            let fill = if i % 2 == 1 {
                ui.visuals().faint_bg_color
            } else {
                egui::Color32::TRANSPARENT
            };
            egui::Frame::new().fill(fill).show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (galley, width) in [(left, status), (right, text_width)] {
                        ui.allocate_ui_with_layout(
                            egui::vec2(width, height),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                ui.set_width(width);
                                ui.add(egui::Label::new(galley));
                            },
                        );
                    }
                });
            });
        }
    });
    m.record(ui, "line-results", &r.response);
}
fn terminal(
    ui: &mut egui::Ui,
    t: &Task,
    s: &mut work::Scope,
    m: &mut Metrics,
    n: &mut Option<String>,
) {
    let h = ui.text_style_height(&egui::TextStyle::Monospace) * 7.;
    let output=egui::Frame::new().fill(palette(ui).input).inner_margin(controls::PAD_X).show(ui,|ui|{
        ui.strong("Actual command output");
        controls::nested_scroll(ui,egui::ScrollArea::both().id_salt(("auth-filter-command-output",&t.id)).auto_shrink([false,false]).min_scrolled_height(h).max_height(h).stick_to_bottom(true),|ui|{
            if s.console.is_empty(){lesson_presentation::prose(ui,"No command has run. Save your filter, then run the supplied test command.");}else{ui.add(egui::Label::new(RichText::new(&s.console).monospace()).extend());}
        });
    });
    m.record(ui, "output-viewport", &output.response);
    let mut run = false;
    let mut field = |ui: &mut egui::Ui, m: &mut Metrics| {
        let r = shared_libraries_page::fitting_input(
            ui,
            &s.command,
            egui::Id::new(("auth-filter-command-input", &t.id)),
            1024,
            ui.is_enabled(),
            "Virtual command · Enter to run",
        );
        m.record(ui, "command", &r.response);
        m.fields
            .push(("command".into(), r.response.rect, r.response.id));
        if let Some(input) = r.value {
            apply(s, t, Action::Command { input }, n);
        }
        run |= ui.is_enabled()
            && (r.response.has_focus() || r.response.lost_focus())
            && ui.input(|i| i.key_pressed(egui::Key::Enter));
        if r.rejected {
            lesson_presentation::prose(
                ui,
                "Keep one virtual command within 1 KiB; the previous input is retained.",
            );
        }
    };
    let width = controls::button_width(ui, "Run command");
    let clicked = if ui.available_width()
        > width + ui.text_style_height(&egui::TextStyle::Body) * 10. + ui.spacing().item_spacing.x
    {
        ui.horizontal(|ui| {
            let w = (ui.available_width() - width - ui.spacing().item_spacing.x).max(1.);
            ui.allocate_ui_with_layout(
                egui::vec2(w, controls::height(ui)),
                egui::Layout::top_down(egui::Align::Min),
                |ui| field(ui, m),
            );
            let r = ui.add(button("Run command"));
            m.record(ui, "run", &r);
            r.clicked()
        })
        .inner
    } else {
        field(ui, m);
        let r = ui.add(button("Run command"));
        m.record(ui, "run", &r);
        r.clicked()
    };
    if clicked || run {
        apply(s, t, Action::Run, n);
    }
}
fn results(ui: &mut egui::Ui, s: &work::Scope, t: &Task, practice: bool, m: &mut Metrics) {
    if !s.checked {
        return;
    }
    let checks = s.checks(t);
    let passed = checks.iter().filter(|c| c.passed).count();
    let r = ui.strong(if checks.len() != t.cases.len() {
        "Filter needs attention · samples cannot be checked yet".to_owned()
    } else if passed == checks.len() {
        if practice {
            "Example checks passed".to_owned()
        } else {
            "Assignment complete".to_owned()
        }
    } else {
        format!("Needs attention · {passed}/{} samples pass", checks.len())
    });
    m.result = Some((passed, checks.len(), r.rect));
    m.record(ui, "result", &r);
    for c in checks.iter().filter(|c| !c.passed) {
        ui.strong(format!("Needs work · {}", c.label));
        let r = lesson_presentation::prose(ui, &c.detail);
        m.record(ui, "failed-case", &r);
    }
    if passed != checks.len() {
        controls::collapsing_header(ui, "Passed samples")
            .id_salt(("auth-filter-passed", &t.id))
            .show(ui, |ui| {
                for c in checks.iter().filter(|c| c.passed) {
                    lesson_presentation::prose(ui, &format!("Pass · {}: {}", c.label, c.detail));
                }
            });
    }
}
impl LabApp {
    pub(super) fn auth_filter_lesson(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let original = &self.progress.attempts[&source.id];
        let d = match binding::source(&source) {
            Ok(d) => d,
            Err(e) => {
                lesson_presentation::prose(ui, &e);
                return;
            }
        };
        if let Err(e) = binding::context(original, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::State::AuthFilter { simulation } = &original.state else {
            return;
        };
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
                    self.metrics.auth_filter.record(ui, key, &r);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        }
        next.view = view;
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            let r = lesson_presentation::concepts(ui, &source, "", Vec::<(String, String)>::new());
            self.metrics
                .auth_filter
                .record(ui, "introduction-content", &r.objective);
            lesson_presentation::worked(
                ui,
                "Worked example · understand which messages the filter selects before editing it.",
            );
            let r = ui.scope(|ui| lesson_presentation::wrapped_worked_commands(ui, &source));
            self.metrics.auth_filter.record(ui, "worked", &r.response);
            let r = ui.add_enabled(active, primary(ui, "Try example"));
            self.metrics.auth_filter.record(ui, "try-example", &r);
            if r.clicked() {
                next.view = teaching::View::Example;
                self.lesson_scroll_reset = true;
            }
        } else {
            let (t, s) = if practice {
                (&d.example, &mut next.example)
            } else {
                (&d.assignment, &mut next.assignment)
            };
            if !practice {
                let r = lesson_presentation::assignment(ui, &source);
                self.metrics
                    .auth_filter
                    .record(ui, "assignment-content", &r);
            }
            ui.add_enabled_ui(active,|ui|{
                if practice{lesson_presentation::practice_step(ui,&source,0);}
                if ui.available_width()>=ui.text_style_height(&egui::TextStyle::Body)*62. {
                    ui.columns(2,|cols|{editor(&mut cols[0],t,s,&mut self.metrics.auth_filter,&mut self.notice);observations(&mut cols[1],t,s,&mut self.metrics.auth_filter,&mut self.notice);});
                }else{editor(ui,t,s,&mut self.metrics.auth_filter,&mut self.notice);ui.add_space(12.);observations(ui,t,s,&mut self.metrics.auth_filter,&mut self.notice);}
                if practice{lesson_presentation::practice_step(ui,&source,1);}
                terminal(ui,t,s,&mut self.metrics.auth_filter,&mut self.notice);
                if practice{lesson_presentation::practice_step(ui,&source,2);}
                let r=ui.add(primary(ui,if practice{"Check example"}else{"Check work"}));self.metrics.auth_filter.record(ui,"check",&r);if r.clicked(){apply(s,t,Action::Check,&mut self.notice);}
                results(ui,s,t,practice,&mut self.metrics.auth_filter);
                if s.checked&&s.checks(t).iter().all(|c|c.passed){lesson_presentation::prose(ui,if practice{"Use the same reasoning independently in the assignment."}else{&source.explanation});}
                if practice {controls::collapsing_header(ui,"Show worked answer · example only").id_salt(("auth-filter-worked",&source.id)).show(ui,|ui|{
                    for body in d.worked.iter().filter_map(|a|match a{Action::Edit{text}=>Some(text),_=>None}){lesson_presentation::monospace(ui,body,true);let r=ui.add(button("Fill worked example draft"));self.metrics.auth_filter.record(ui,"fill-worked",&r);if r.clicked(){apply(s,t,Action::Edit{text:body.clone()},&mut self.notice);}}
                    lesson_presentation::prose(ui,"This fills only the example editor. Save virtual filter, then Run command to test it.");
                });}
            });
            if practice {
                let r = ui.add_enabled(active, button("Reset example only"));
                self.metrics.auth_filter.record(ui, "reset-example", &r);
                if r.clicked()
                    && let Err(e) = next.reset_example(d)
                {
                    self.notice = Some(e);
                }
            }
            controls::collapsing_header(ui, "Help, supported subset and references")
                .id_salt(("auth-filter-help", &source.id))
                .show(ui, |ui| {
                    if ui.add_enabled(active, button("Show a hint")).clicked() {
                        let a = self.progress.attempts.get_mut(&source.id).unwrap();
                        a.hints = (a.hints + 1).min(source.hints.len());
                        self.dirty = true;
                    }
                    for hint in source
                        .hints
                        .iter()
                        .take(self.progress.attempts[&source.id].hints)
                    {
                        lesson_presentation::prose(ui, hint);
                    }
                    controls::collapsing_header(
                        ui,
                        "Worked approach · reveals assignment guidance",
                    )
                    .id_salt(("auth-filter-assignment-worked", &source.id))
                    .show(ui, |ui| {
                        lesson_presentation::prose(ui, &source.explanation);
                    });
                    lesson_presentation::prose(ui, binding::HELP);
                    lesson_presentation::prose(ui, &source.simplifications);
                    for url in &source.sources {
                        ui.hyperlink_to(url, url);
                    }
                });
        }
        if active && next != before {
            match d.bound(&next) {
                Ok(()) => {
                    self.progress.attempts.get_mut(&source.id).unwrap().state =
                        engine::State::AuthFilter {
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
        Self::workspace_footer(ui);
    }
}
