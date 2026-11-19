//! Capacity investigation using a virtual file table and native logical-size editor.
use super::*;
use dlms_learning_lab::capacity_workspace as work;
#[derive(Default)]
pub(super) struct Metrics {
    pub controls: Vec<(String, egui::Rect, egui::Rect)>,
    pub fields: Vec<(String, egui::Rect, egui::Id)>,
    pub result: Option<(usize, usize, egui::Rect)>,
}
fn record(m: &mut Metrics, key: &str, r: &egui::Response, ui: &egui::Ui) {
    m.controls.push((key.into(), r.rect, ui.clip_rect()));
}
fn usage(ui: &mut egui::Ui, w: &dlms_learning_lab::fs_capacity::State) {
    let u = w.usage().expect("validated capacity world");
    let p = w.profile().unwrap();
    ui.add_space(10.);
    lesson_presentation::rich(ui, RichText::new("Current capacity").strong());
    for (label, value) in [
        ("Block size", format!("{} bytes", u.block_bytes)),
        (
            "Blocks used / total",
            format!("{} / {}", u.used_blocks, u.total_blocks),
        ),
        ("Blocks free", u.free_blocks.to_string()),
        ("Reserved blocks", p.root_reserved_blocks.to_string()),
        (
            "Available to ordinary files",
            format!(
                "{} blocks · {} bytes",
                u.available_blocks, u.available_bytes
            ),
        ),
        (
            "Free inodes",
            format!("{} of {}", u.free_inodes, u.total_inodes),
        ),
    ] {
        controls::form_row(ui, label, |ui, _width| {
            lesson_presentation::rich(ui, RichText::new(value).monospace());
        });
    }
}
fn editor(ui: &mut egui::Ui, s: &mut work::Scope, m: &mut Metrics, active: bool) {
    ui.add_space(10.);
    lesson_presentation::rich(
        ui,
        RichText::new("Create or resize a virtual file").strong(),
    );
    lesson_presentation::prose(
        ui,
        "Select Edit beside an existing file, or enter a new /name. Logical bytes are the file’s length; the capacity table shows its rounded block use.",
    );
    controls::form_row(ui, "Virtual path", |ui, width| {
        let r = ui.add_enabled(
            active,
            controls::singleline(ui, &mut s.path)
                .desired_width((width.min(310.) - 2. * f32::from(controls::PAD_X)).max(1.))
                .hint_text("/name"),
        );
        m.fields.push(("path".into(), r.rect, r.id));
        record(m, "path", &r, ui);
        if r.changed() {
            s.dirty = true;
            s.checked = false;
        }
    });
    controls::form_row(ui, "Logical bytes", |ui, width| {
        let r = ui.add_enabled(
            active,
            controls::singleline(ui, &mut s.bytes)
                .desired_width((width.min(310.) - 2. * f32::from(controls::PAD_X)).max(1.))
                .hint_text("Whole number of bytes"),
        );
        m.fields.push(("bytes".into(), r.rect, r.id));
        record(m, "bytes", &r, ui);
        if r.changed() {
            s.dirty = true;
            s.checked = false;
        }
    });
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(active, button("Apply file length"));
        record(m, "put", &r, ui);
        if r.clicked()
            && let Err(e) = s.put()
        {
            s.checked = false;
            s.message = e;
        }
        let r = ui.add_enabled(active && s.dirty, button("Discard draft"));
        record(m, "discard", &r, ui);
        if r.clicked() {
            s.path.clear();
            s.bytes.clear();
            s.dirty = false;
            s.checked = false;
            s.message = "Discarded only the unapplied editor draft.".into();
        }
    });
}
fn files(ui: &mut egui::Ui, s: &mut work::Scope, m: &mut Metrics, active: bool) {
    let b = s.world.usage().unwrap().block_bytes;
    let retained = &s.world.profile().unwrap().retained;
    let rows = s.world.files().unwrap().clone();
    ui.add_space(10.);
    lesson_presentation::rich(ui, RichText::new("Virtual files").strong());
    controls::nested_scroll(
        ui,
        egui::ScrollArea::horizontal().id_salt("capacity-files"),
        |ui| {
            egui::Grid::new("capacity-files-table")
                .num_columns(5)
                .spacing(egui::vec2(14., 6.))
                .striped(true)
                .show(ui, |ui| {
                    for title in ["Remove", "Path", "Logical bytes", "Blocks", "Action"] {
                        ui.strong(title);
                    }
                    ui.end_row();
                    for (path, bytes) in rows {
                        let protected = retained.contains_key(&path);
                        let mut selected = s.selected.contains(&path);
                        let r = ui.add_enabled(
                            active && !protected,
                            egui::Checkbox::without_text(&mut selected),
                        );
                        record(m, &format!("select-{path}"), &r, ui);
                        if r.changed() {
                            if selected {
                                s.selected.push(path.clone());
                            } else {
                                s.selected.retain(|p| p != &path);
                            }
                        }
                        ui.monospace(&path);
                        ui.monospace(bytes.to_string());
                        ui.monospace(bytes.div_ceil(b).to_string());
                        if protected {
                            ui.label("Retain unchanged");
                        } else {
                            let r = ui.add_enabled(active, button("Edit"));
                            record(m, &format!("edit-{path}"), &r, ui);
                            if r.clicked() {
                                s.path = path.clone();
                                s.bytes = bytes.to_string();
                                s.dirty = false;
                                s.checked = false;
                            }
                        }
                        ui.end_row();
                    }
                });
        },
    );
    ui.horizontal_wrapped(|ui| {
        let r = ui.add_enabled(active && !s.selected.is_empty(), button("Preview removal"));
        record(m, "preview", &r, ui);
        if r.clicked()
            && let Err(e) = s.preview_remove()
        {
            s.message = e;
        }
        let r = ui.add_enabled(active && s.preview.is_some(), button("Apply removal"));
        record(m, "remove", &r, ui);
        if r.clicked()
            && let Err(e) = s.apply_remove()
        {
            s.message = e;
        }
    });
    if let Some(p) = &s.preview {
        let names: Vec<_> = p.names.iter().map(String::as_str).collect();
        let plan = p.before.preview_remove(&names).unwrap();
        let u = plan.projected();
        lesson_presentation::prose(
            ui,
            &format!(
                "Preview · remove {}. Projected: {} ordinary-available blocks, {} free inodes. {}",
                p.names.join(", "),
                u.available_blocks,
                u.free_inodes,
                if p.before == s.world {
                    "Matches current files."
                } else {
                    "Stale: files changed. Preview again before applying."
                }
            ),
        );
    }
}
impl LabApp {
    pub(super) fn capacity_page(&mut self, ui: &mut egui::Ui, i: usize) {
        let phase = (self.selected, self.reset_pending);
        let challenge = self.workspace_header(ui, i);
        if phase != (self.selected, self.reset_pending) {
            return;
        }
        let source = self.scenarios[i].clone();
        let a = &self.progress.attempts[&source.id];
        if let Err(e) = work::context(a, &source) {
            lesson_presentation::prose(ui, &e);
            return;
        }
        let engine::Goal::Capacity { definition } = &source.goal else {
            return;
        };
        let engine::State::Capacity { simulation } = &a.state else {
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
                    let r = ui.add_enabled(active, egui::Button::new(label).selected(view == v));
                    record(&mut self.metrics.capacity, key, &r, ui);
                    self.metrics.lesson_tabs.push(r.rect);
                    if r.clicked() {
                        view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
            next.view = view;
        }
        let practice = view == teaching::View::Example;
        if view == teaching::View::Introduction {
            lesson_presentation::concepts(
                ui,
                &source,
                "Builds on files, paths and storage reports.",
                Vec::<(String, String)>::new(),
            );
            lesson_presentation::worked(
                ui,
                "Different sample files demonstrate the reasoning. Apply these actions in Try example; they do not change the assignment files.",
            );
            lesson_presentation::worked_rows(ui, &source);
            ui.horizontal_wrapped(|ui| {
                for (v, key, label) in [
                    (teaching::View::Example, "try", "Try this example"),
                    (teaching::View::Assignment, "start", "Open full assignment"),
                ] {
                    let r = ui.add_enabled(active, button(label));
                    record(&mut self.metrics.capacity, key, &r, ui);
                    if r.clicked() {
                        next.view = v;
                        self.lesson_scroll_reset = true;
                    }
                }
            });
        } else {
            if practice {
                lesson_presentation::practice_step(ui, &source, 0);
            } else {
                lesson_presentation::assignment(ui, &source);
            }
            let s = if practice {
                &mut next.practice
            } else {
                &mut next.assignment
            };
            usage(ui, &s.world);
            if practice {
                lesson_presentation::practice_step(ui, &source, 1);
                let a = &definition.example_actions[next.example_action];
                lesson_presentation::prose(
                    ui,
                    &match a {
                        work::Action::Put { path, bytes } => format!(
                            "Worked action: enter {path} with {bytes} logical bytes, then Apply file length."
                        ),
                        work::Action::Remove { names } => format!(
                            "Worked action: select {}, Preview removal, inspect the projection, then Apply removal.",
                            names.join(", ")
                        ),
                    },
                );
                ui.horizontal_wrapped(|ui| {
                    let r = ui.add_enabled(active, button("Prepare worked action"));
                    record(&mut self.metrics.capacity, "fill-example", &r, ui);
                    if r.clicked() {
                        match a {
                            work::Action::Put { path, bytes } => {
                                s.path = path.clone();
                                s.bytes = bytes.to_string();
                                s.dirty = true;
                                s.checked = false;
                            }
                            work::Action::Remove { names } => {
                                s.selected = names.clone();
                                s.preview = None;
                            }
                        }
                    }
                    let r = ui.add_enabled(
                        active && next.example_action + 1 < definition.example_actions.len(),
                        button("Next worked action"),
                    );
                    record(&mut self.metrics.capacity, "next-action", &r, ui);
                    if r.clicked() {
                        next.example_action += 1;
                    }
                });
            }
            // Preserve native tables and editors. At narrow widths stack rather than squeeze.
            let wide = ui.available_width() > 970. * self.scale;
            if wide {
                ui.columns(2, |columns| {
                    columns[0].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        files(ui, s, &mut self.metrics.capacity, active)
                    });
                    columns[1].with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                        editor(ui, s, &mut self.metrics.capacity, active)
                    });
                });
            } else {
                files(ui, s, &mut self.metrics.capacity, active);
                editor(ui, s, &mut self.metrics.capacity, active);
            }
            if !s.message.is_empty() {
                lesson_presentation::prose(ui, &s.message);
            }
            let r = ui.add_enabled(
                active,
                primary(
                    ui,
                    if practice {
                        "Check example"
                    } else {
                        "Check capacity"
                    },
                ),
            );
            record(&mut self.metrics.capacity, "check", &r, ui);
            self.metrics.check = Some(r.rect);
            if r.clicked() {
                s.checked = true;
            }
            if s.checked {
                let rows = definition.checks(&next, practice);
                let failed = rows.iter().filter(|r| !r.0).count();
                let r = lesson_presentation::rich(
                    ui,
                    RichText::new(if failed == 0 {
                        format!("Passed · all {} outcome checks", rows.len())
                    } else {
                        format!("Needs attention · {failed} of {} checks", rows.len())
                    })
                    .strong(),
                );
                self.metrics.capacity.result = Some((rows.len(), failed, r.rect));
                for (ok, text) in rows {
                    lesson_presentation::rich(
                        ui,
                        RichText::new(format!(
                            "{} · {text}",
                            if ok { "Passed" } else { "Needs attention" }
                        ))
                        .color(if ok {
                            palette(ui).success
                        } else {
                            palette(ui).error
                        }),
                    );
                }
            } else {
                lesson_presentation::prose(
                    ui,
                    "Not checked · inspect current capacity, then use Check.",
                );
            }
            if practice {
                lesson_presentation::practice_step(ui, &source, 2);
                let r = ui.add_enabled(active, button("Reset example only"));
                record(&mut self.metrics.capacity, "reset-example", &r, ui);
                if r.clicked() {
                    next.practice = work::Scope::new(&definition.example);
                    next.example_action = 0;
                }
            }
            controls::collapsing_header(ui, "Help and model limits").show(ui, |ui| {
                lesson_presentation::prose(ui, work::HELP);
                if !practice {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    let r = ui.add_enabled(
                        active && a.hints < source.hints.len(),
                        button("Show a hint"),
                    );
                    if r.clicked() {
                        a.hints += 1;
                        self.dirty = true;
                    }
                    for hint in source.hints.iter().take(a.hints) {
                        lesson_presentation::prose(ui, hint);
                    }
                    controls::collapsing_header(ui, "Worked approach · reveals a solution").show(
                        ui,
                        |ui| {
                            lesson_presentation::prose(ui, &source.explanation);
                        },
                    );
                }
                for link in &source.sources {
                    ui.hyperlink_to(link, link);
                }
            });
        }
        if next != before && active {
            match definition.bound(&next) {
                Ok(()) => {
                    let a = self.progress.attempts.get_mut(&source.id).unwrap();
                    a.state = engine::State::Capacity {
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
    }
}
